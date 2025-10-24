use pkg::recipes;
use pkg::{Package, PackageName};
use redoxer::target;

use crate::cook::fs::*;
use crate::cook::script::*;
use crate::recipe::AutoDeps;
use crate::recipe::BuildKind;
use crate::recipe::Recipe;
use std::collections::VecDeque;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
    str,
    time::SystemTime,
};

use crate::is_redox;

use crate::REMOTE_PKG_SOURCE;

fn auto_deps(
    stage_dir: &Path,
    dep_pkgars: &BTreeSet<(PackageName, PathBuf)>,
) -> BTreeSet<PackageName> {
    let mut paths = BTreeSet::new();
    let mut visited = BTreeSet::new();
    // Base directories may need to be updated for packages that place binaries in odd locations.
    let mut walk = VecDeque::from([
        stage_dir.join("libexec"),
        stage_dir.join("usr/bin"),
        stage_dir.join("usr/games"),
        stage_dir.join("usr/lib"),
        stage_dir.join("usr/libexec"),
    ]);

    // Recursively (DFS) walk each directory to ensure nested libs and bins are checked.
    while let Some(dir) = walk.pop_front() {
        let Ok(dir) = dir.canonicalize() else {
            continue;
        };
        if visited.contains(&dir) {
            #[cfg(debug_assertions)]
            eprintln!("DEBUG: auto_deps => Skipping `{dir:?}` (already visited)");
            continue;
        }
        assert!(
            visited.insert(dir.clone()),
            "Directory `{:?}` should not be in visited\nVisited: {:#?}",
            dir,
            visited
        );

        let Ok(read_dir) = fs::read_dir(&dir) else {
            continue;
        };
        for entry_res in read_dir {
            let Ok(entry) = entry_res else { continue };
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_file() {
                paths.insert(entry.path());
            } else if file_type.is_dir() {
                walk.push_front(entry.path());
            }
        }
    }

    let mut needed = BTreeSet::new();
    for path in paths {
        let Ok(file) = fs::File::open(&path) else {
            continue;
        };
        let read_cache = object::ReadCache::new(file);
        let Ok(object) = object::build::elf::Builder::read(&read_cache) else {
            continue;
        };
        let Some(dynamic_data) = object.dynamic_data() else {
            continue;
        };
        for dynamic in dynamic_data {
            let object::build::elf::Dynamic::String { tag, val } = dynamic else {
                continue;
            };
            if *tag == object::elf::DT_NEEDED {
                let Ok(name) = str::from_utf8(val) else {
                    continue;
                };
                if let Ok(relative_path) = path.strip_prefix(stage_dir) {
                    eprintln!("DEBUG: {} needs {}", relative_path.display(), name);
                }
                needed.insert(name.to_string());
            }
        }
    }

    let mut missing = needed.clone();
    // relibc and friends will always be installed
    for preinstalled in &["libc.so.6", "libgcc_s.so.1", "libstdc++.so.6"] {
        missing.remove(*preinstalled);
    }

    let mut deps = BTreeSet::new();
    if let Ok(key_file) = pkgar_keys::PublicKeyFile::open("build/id_ed25519.pub.toml") {
        for (dep, archive_path) in dep_pkgars.iter() {
            let Ok(mut package) = pkgar::PackageFile::new(archive_path, &key_file.pkey) else {
                continue;
            };
            let Ok(entries) = pkgar_core::PackageSrc::read_entries(&mut package) else {
                continue;
            };
            for entry in entries {
                let Ok(entry_path) = pkgar::ext::EntryExt::check_path(&entry) else {
                    continue;
                };
                for prefix in &["lib", "usr/lib"] {
                    let Ok(child_path) = entry_path.strip_prefix(prefix) else {
                        continue;
                    };
                    let Some(child_name) = child_path.to_str() else {
                        continue;
                    };
                    if needed.contains(child_name) {
                        eprintln!("DEBUG: {} provides {}", dep, child_name);
                        deps.insert(dep.clone());
                        missing.remove(child_name);
                    }
                }
            }
        }
    }

    for name in missing {
        eprintln!("WARN: {} missing", name);
    }

    deps
}

pub fn build(
    recipe_dir: &Path,
    source_dir: &Path,
    target_dir: &Path,
    name: &PackageName,
    recipe: &Recipe,
    offline_mode: bool,
    check_source: bool,
    logger: &Stdout,
) -> Result<(PathBuf, BTreeSet<PackageName>), String> {
    let sysroot_dir = target_dir.join("sysroot");
    let stage_dir = target_dir.join("stage");
    if recipe.build.kind == BuildKind::None {
        // metapackages don't need to do anything here
        return Ok((stage_dir, BTreeSet::new()));
    }

    let mut dep_pkgars = BTreeSet::new();
    for dependency in recipe.build.dependencies.iter() {
        let dependency_dir = recipes::find(dependency.as_str());
        if dependency_dir.is_none() {
            return Err(format!("failed to find recipe directory '{}'", dependency));
        }
        dep_pkgars.insert((
            dependency.clone(),
            dependency_dir
                .unwrap()
                .join("target")
                .join(redoxer::target())
                .join("stage.pkgar"),
        ));
    }

    if stage_dir.exists() && !check_source {
        let auto_deps = build_auto_deps(target_dir, &stage_dir, dep_pkgars)?;
        return Ok((stage_dir, auto_deps));
    }

    let source_modified = modified_dir_ignore_git(source_dir)?;
    let deps_modified = dep_pkgars
        .iter()
        .map(|(_dep, pkgar)| modified(pkgar))
        .max()
        .unwrap_or(Ok(SystemTime::UNIX_EPOCH))?;

    // Rebuild sysroot if source is newer
    //TODO: rebuild on recipe changes
    if sysroot_dir.is_dir() {
        let sysroot_modified = modified_dir(&sysroot_dir)?;
        if sysroot_modified < source_modified || sysroot_modified < deps_modified {
            eprintln!(
                "DEBUG: '{}' newer than '{}'",
                source_dir.display(),
                sysroot_dir.display()
            );
            remove_all(&sysroot_dir)?;
        }
    }
    if !sysroot_dir.is_dir() {
        // Create sysroot.tmp
        let sysroot_dir_tmp = target_dir.join("sysroot.tmp");
        create_dir_clean(&sysroot_dir_tmp)?;

        // Make sure sysroot/usr exists
        create_dir(&sysroot_dir_tmp.join("usr"))?;
        for folder in &["bin", "include", "lib", "share"] {
            // Make sure sysroot/usr/$folder exists
            create_dir(&sysroot_dir_tmp.join("usr").join(folder))?;

            // Link sysroot/$folder sysroot/usr/$folder
            symlink(Path::new("usr").join(folder), &sysroot_dir_tmp.join(folder))?;
        }

        for (_dep, archive_path) in &dep_pkgars {
            let public_path = "build/id_ed25519.pub.toml";
            pkgar::extract(
                public_path,
                &archive_path,
                sysroot_dir_tmp.to_str().unwrap(),
            )
            .map_err(|err| {
                format!(
                    "failed to install '{}' in '{}': {:?}",
                    archive_path.display(),
                    sysroot_dir_tmp.display(),
                    err
                )
            })?;
        }

        // Move sysroot.tmp to sysroot atomically
        rename(&sysroot_dir_tmp, &sysroot_dir)?;
    }

    // Rebuild stage if source is newer
    //TODO: rebuild on recipe changes
    if stage_dir.is_dir() {
        let stage_modified = modified_dir(&stage_dir)?;
        if stage_modified < source_modified || stage_modified < deps_modified {
            eprintln!(
                "DEBUG: '{}' newer than '{}'",
                source_dir.display(),
                stage_dir.display()
            );
            remove_all(&stage_dir)?;
        }
    }

    if !stage_dir.is_dir() {
        // Create stage.tmp
        let stage_dir_tmp = target_dir.join("stage.tmp");
        create_dir_clean(&stage_dir_tmp)?;

        // Create build, if it does not exist
        //TODO: flag for clean builds where build is wiped out
        let build_dir = target_dir.join("build");
        if !build_dir.is_dir() {
            create_dir_clean(&build_dir)?;
        }

        let flags_fn = |name, flags: &Vec<String>| {
            format!(
                "{name}+=(\n{}\n)\n",
                flags
                    .iter()
                    .map(|s| format!("  \"{s}\""))
                    .collect::<Vec<String>>()
                    .join("\n")
            )
        };

        //TODO: better integration with redoxer (library instead of binary)
        //TODO: configurable target
        //TODO: Add more configurability, convert scripts to Rust?
        let script = match &recipe.build.kind {
            BuildKind::Cargo {
                package_path,
                cargoflags,
            } => {
                format!(
                    "PACKAGE_PATH={} cookbook_cargo {cargoflags}",
                    package_path.as_deref().unwrap_or(".")
                )
            }
            BuildKind::Configure { configureflags } => format!(
                "DYNAMIC_INIT\n{}cookbook_configure",
                flags_fn("COOKBOOK_CONFIGURE_FLAGS", configureflags),
            ),
            BuildKind::Cmake { cmakeflags } => format!(
                "DYNAMIC_INIT\n{}cookbook_cmake",
                flags_fn("COOKBOOK_CMAKE_FLAGS", cmakeflags),
            ),
            BuildKind::Meson { mesonflags } => format!(
                "DYNAMIC_INIT\n{}cookbook_meson",
                flags_fn("COOKBOOK_MESON_FLAGS", mesonflags),
            ),
            BuildKind::Custom { script } => script.clone(),
            BuildKind::Remote => return build_remote(target_dir, name, offline_mode, logger),
            BuildKind::None => "".to_owned(),
        };

        let command = {
            //TODO: remove unwraps
            let cookbook_build = build_dir.canonicalize().unwrap();
            let cookbook_recipe = recipe_dir.canonicalize().unwrap();
            let cookbook_root = Path::new(".").canonicalize().unwrap();
            let cookbook_stage = stage_dir_tmp.canonicalize().unwrap();
            let cookbook_source = source_dir.canonicalize().unwrap();
            let cookbook_sysroot = sysroot_dir.canonicalize().unwrap();

            let mut command = if is_redox() {
                let mut command = Command::new("bash");
                command.arg("-ex");
                command.env("COOKBOOK_REDOXER", "cargo");
                command
            } else {
                let cookbook_redoxer = Path::new("target/release/cookbook_redoxer")
                    .canonicalize()
                    .unwrap();
                let mut command = Command::new(&cookbook_redoxer);
                command.arg("env").arg("bash").arg("-ex");
                command.env("COOKBOOK_REDOXER", &cookbook_redoxer);
                command
            };
            command.current_dir(&cookbook_build);
            command.env("COOKBOOK_BUILD", &cookbook_build);
            command.env("COOKBOOK_NAME", name.as_str());
            command.env("COOKBOOK_RECIPE", &cookbook_recipe);
            command.env("COOKBOOK_ROOT", &cookbook_root);
            command.env("COOKBOOK_STAGE", &cookbook_stage);
            command.env("COOKBOOK_SOURCE", &cookbook_source);
            command.env("COOKBOOK_SYSROOT", &cookbook_sysroot);
            if offline_mode {
                command.env("COOKBOOK_OFFLINE", "1");
            }
            command
        };

        let full_script = format!(
            "{}\n{}\n{}\n{}",
            BUILD_PRESCRIPT, SHARED_PRESCRIPT, script, BUILD_POSTSCRIPT
        );
        run_command_stdin(command, full_script.as_bytes(), logger)?;

        // Move stage.tmp to stage atomically
        rename(&stage_dir_tmp, &stage_dir)?;
    }

    let auto_deps = build_auto_deps(target_dir, &stage_dir, dep_pkgars)?;

    Ok((stage_dir, auto_deps))
}

/// Calculate automatic dependencies
fn build_auto_deps(
    target_dir: &Path,
    stage_dir: &PathBuf,
    dep_pkgars: BTreeSet<(PackageName, PathBuf)>,
) -> Result<BTreeSet<PackageName>, String> {
    let auto_deps_path = target_dir.join("auto_deps.toml");
    if auto_deps_path.is_file() && modified(&auto_deps_path)? < modified(stage_dir)? {
        remove_all(&auto_deps_path)?
    }

    let auto_deps = if auto_deps_path.exists() {
        let toml_content =
            fs::read_to_string(&auto_deps_path).map_err(|_| "failed to read cached auto_deps")?;
        let wrapper: AutoDeps =
            toml::from_str(&toml_content).map_err(|_| "failed to deserialize cached auto_deps")?;
        wrapper.packages
    } else {
        let packages = auto_deps(stage_dir, &dep_pkgars);
        let wrapper = AutoDeps { packages };
        serialize_and_write(&auto_deps_path, &wrapper)?;
        wrapper.packages
    };
    Ok(auto_deps)
}

fn get_remote_url(name: &PackageName, ext: &str) -> String {
    return format!("{}/{}/{}.{}", REMOTE_PKG_SOURCE, target(), name, ext);
}
fn get_pubkey_url() -> String {
    return format!("{}/id_ed25519.pub.toml", REMOTE_PKG_SOURCE);
}

pub fn build_remote(
    target_dir: &Path,
    name: &PackageName,
    offline_mode: bool,
    logger: &Stdout,
) -> Result<(PathBuf, BTreeSet<PackageName>), String> {
    // download straight from remote source then declare pkg dependencies as autodeps dependency
    let stage_dir = target_dir.join("stage");

    let source_pkgar = target_dir.join("source.pkgar");
    let source_toml = target_dir.join("source.toml");
    let source_pubkey = target_dir.join("id_ed25519.pub.toml");

    if !offline_mode {
        download_wget(&get_remote_url(name, "pkgar"), &source_pkgar, logger)?;
        download_wget(&get_remote_url(name, "toml"), &source_toml, logger)?;
        download_wget(&get_pubkey_url(), &source_pubkey, logger)?;
    } else {
        offline_check_exists(&source_pkgar)?;
        offline_check_exists(&source_toml)?;
        offline_check_exists(&source_pubkey)?;
    }

    if stage_dir.is_dir() && modified(&source_pkgar)? > modified(&stage_dir)? {
        remove_all(&stage_dir)?
    }
    if !stage_dir.is_dir() {
        let stage_dir_tmp = target_dir.join("stage.tmp");

        pkgar::extract(&source_pubkey, &source_pkgar, &stage_dir_tmp).map_err(|err| {
            format!(
                "failed to install '{}' in '{}': {:?}",
                source_pkgar.display(),
                stage_dir_tmp.display(),
                err
            )
        })?;

        // Move stage.tmp to stage atomically
        rename(&stage_dir_tmp, &stage_dir)?;
    }

    let auto_deps_path = target_dir.join("auto_deps.toml");
    if auto_deps_path.is_file() && modified(&auto_deps_path)? < modified(&stage_dir)? {
        remove_all(&auto_deps_path)?
    }

    let auto_deps = if auto_deps_path.exists() {
        let toml_content =
            fs::read_to_string(&auto_deps_path).map_err(|_| "failed to read cached auto_deps")?;
        let wrapper: AutoDeps =
            toml::from_str(&toml_content).map_err(|_| "failed to deserialize cached auto_deps")?;
        wrapper.packages
    } else {
        let toml_content =
            fs::read_to_string(&source_toml).map_err(|_| "failed to read source.toml")?;
        let pkg_toml: Package =
            toml::from_str(&toml_content).map_err(|_| "failed to deserialize source.toml")?;
        let wrapper = AutoDeps {
            packages: pkg_toml.depends.into_iter().collect(),
        };
        serialize_and_write(&auto_deps_path, &wrapper)?;
        wrapper.packages
    };

    Ok((stage_dir, auto_deps))
}

#[cfg(test)]
mod tests {
    use std::os::unix;

    use super::auto_deps;

    #[test]
    fn file_system_loop_no_infinite_loop() {
        // Hierarchy with an infinite loop
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let dir = root.join("loop");
        unix::fs::symlink(root, &dir).expect("Linking {dir:?} to {root:?}");

        // Sanity check that we have a loop
        assert_eq!(
            root.canonicalize().unwrap(),
            dir.canonicalize().unwrap(),
            "Expected a loop where {dir:?} points to {root:?}"
        );

        let entries = auto_deps(root, &Default::default());
        assert!(
            entries.is_empty(),
            "auto_deps shouldn't have yielded any libraries"
        );
    }
}
