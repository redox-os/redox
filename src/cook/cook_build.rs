use pkg::package::PackageError;
use pkg::{Package, PackageName};

use crate::cook::fs::*;
use crate::cook::package::{package_source_paths, package_target};
use crate::cook::pty::PtyOut;
use crate::cook::script::*;
use crate::recipe::Recipe;
use crate::recipe::{AutoDeps, CookRecipe};
use crate::recipe::{BuildKind, OptionalPackageRecipe};
use std::collections::VecDeque;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
    str,
    time::SystemTime,
};

use crate::{is_redox, log_to_pty};

fn auto_deps_from_dynamic_linking(
    stage_dirs: &Vec<PathBuf>,
    target_dir: &Path,
    dep_pkgars: &BTreeSet<(PackageName, PathBuf)>,
    logger: &PtyOut,
) -> BTreeSet<PackageName> {
    let mut paths = BTreeSet::new();
    let mut visited = BTreeSet::new();
    let verbose = crate::config::get_config().cook.verbose;
    // Base directories may need to be updated for packages that place binaries in odd locations.
    let mut walk = VecDeque::new();

    for stage_dir in stage_dirs {
        walk.push_back(stage_dir.join("usr/bin"));
        walk.push_back(stage_dir.join("usr/games"));
        walk.push_back(stage_dir.join("usr/lib"));
        walk.push_back(stage_dir.join("usr/libexec"));
    }

    // Recursively (DFS) walk each directory to ensure nested libs and bins are checked.
    while let Some(dir) = walk.pop_front() {
        let Ok(dir) = dir.canonicalize() else {
            continue;
        };
        if visited.contains(&dir) {
            #[cfg(debug_assertions)]
            log_to_pty!(
                logger,
                "DEBUG: auto_deps => Skipping `{dir:?}` (already visited)"
            );
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
                if let Ok(relative_path) = path.strip_prefix(target_dir) {
                    if verbose {
                        log_to_pty!(logger, "DEBUG: {} needs {}", relative_path.display(), name);
                    }
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
                        if verbose {
                            log_to_pty!(logger, "DEBUG: {} provides {}", dep, child_name);
                        }
                        deps.insert(dep.clone());
                        missing.remove(child_name);
                    }
                }
            }
        }
    }

    if verbose {
        for name in missing {
            log_to_pty!(logger, "INFO: {} missing", name);
        }
    }

    deps
}

fn auto_deps_from_static_package_deps(
    build_dep_pkgars: &BTreeSet<(PackageName, PathBuf)>,
    dynamic_dep_pkgars: &BTreeSet<PackageName>,
) -> Result<BTreeSet<PackageName>, PackageError> {
    let static_dep_pkgars: Vec<PackageName> = build_dep_pkgars
        .iter()
        .map(|x| x.0.clone())
        .filter(|x| !dynamic_dep_pkgars.contains(x))
        .collect();
    let pkgs = CookRecipe::get_package_deps_recursive(&static_dep_pkgars, false)?;

    Ok(pkgs.into_iter().collect())
}

pub fn build(
    recipe_dir: &Path,
    source_dir: &Path,
    target_dir: &Path,
    name: &PackageName,
    recipe: &Recipe,
    offline_mode: bool,
    check_source: bool,
    logger: &PtyOut,
) -> Result<(Vec<PathBuf>, BTreeSet<PackageName>), String> {
    let sysroot_dir = target_dir.join("sysroot");
    let toolchain_dir = target_dir.join("toolchain");
    let stage_dirs = get_stage_dirs(&recipe.optional_packages, target_dir);
    let cli_verbose = crate::config::get_config().cook.verbose;
    let cli_jobs = crate::config::get_config().cook.jobs;
    if recipe.build.kind == BuildKind::None {
        // metapackages don't need to do anything here
        return Ok((stage_dirs, BTreeSet::new()));
    }

    let mut dep_pkgars = BTreeSet::new();
    let mut dep_host_pkgars = BTreeSet::new();
    let mut build_deps =
        CookRecipe::get_build_deps_recursive(&recipe.build.dependencies, false, false)
            .map_err(|e| format!("{:?}", e))?;
    for dep in &recipe.build.dev_dependencies {
        build_deps.push(CookRecipe::from_name(dep.clone()).map_err(|e| format!("{:?}", e))?);
    }
    for dependency in build_deps.iter() {
        let (_, pkgar, _) = dependency.stage_paths();
        if dependency.name.is_host() {
            dep_host_pkgars.insert((dependency.name.clone(), pkgar));
        } else {
            dep_pkgars.insert((dependency.name.clone(), pkgar));
        }
    }

    if !check_source && stage_dirs.iter().all(|dir| dir.exists()) {
        let auto_deps = build_auto_deps(recipe, target_dir, &stage_dirs, dep_pkgars, logger)?;
        return Ok((stage_dirs, auto_deps));
    }

    let mut source_modified = modified_dir_ignore_git(source_dir).unwrap_or(SystemTime::UNIX_EPOCH);
    if let Ok(recipe_modified) = modified(&recipe_dir.join("recipe.toml")) {
        if recipe_modified > source_modified {
            source_modified = recipe_modified
        }
    }
    let deps_modified = dep_pkgars
        .iter()
        .map(|(_dep, pkgar)| modified(pkgar))
        .max()
        .unwrap_or(Ok(SystemTime::UNIX_EPOCH))?;
    let deps_host_modified = dep_host_pkgars
        .iter()
        .map(|(_dep, pkgar)| modified(pkgar))
        .max()
        .unwrap_or(Ok(SystemTime::UNIX_EPOCH))?;

    // Rebuild sysroot if source is newer
    if recipe.build.kind != BuildKind::Remote {
        build_deps_dir(
            logger,
            &sysroot_dir,
            target_dir.join("sysroot.tmp"),
            if name.is_host() {
                &dep_host_pkgars
            } else {
                &dep_pkgars
            },
            source_modified,
            deps_modified,
        )?;
    }
    if recipe.build.kind != BuildKind::Remote && !name.is_host() && dep_host_pkgars.len() > 0 {
        build_deps_dir(
            logger,
            &toolchain_dir,
            target_dir.join("toolchain.tmp"),
            &dep_host_pkgars,
            source_modified,
            deps_host_modified,
        )?;
    }

    // Rebuild stage if source is newer
    if stage_dirs.iter().any(|dir| dir.is_dir()) {
        let stage_modified =
            modified_all(&stage_dirs, modified_dir).unwrap_or(SystemTime::UNIX_EPOCH);
        if stage_modified < source_modified
            || stage_modified < deps_modified
            || stage_modified < deps_host_modified
        {
            for stage_dir in &stage_dirs {
                log_to_pty!(logger, "DEBUG: updating '{}'", stage_dir.display());
                if stage_dir.is_dir() {
                    remove_all(&stage_dir)?;
                }
            }
        }
    }

    if !stage_dirs.last().is_some_and(|dir| dir.is_dir()) {
        let stage_dir = stage_dirs
            .last()
            .expect("Should have atleast one stage dir");
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

        if recipe.build.kind == BuildKind::Remote {
            return build_remote(stage_dirs, recipe, target_dir);
        }
        //TODO: better integration with redoxer (library instead of binary)
        //TODO: configurable target
        //TODO: Add more configurability, convert scripts to Rust?
        let script = match &recipe.build.kind {
            BuildKind::Cargo {
                package_path,
                cargoflags,
            } => {
                format!(
                    "DYNAMIC_INIT\nPACKAGE_PATH={} cookbook_cargo {cargoflags}",
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
            BuildKind::Remote => unreachable!(),
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
            let cookbook_toolchain = toolchain_dir.canonicalize().ok();
            let bash_args = if cli_verbose { "-ex" } else { "-e" };
            let mut command = if is_redox() {
                let mut command = Command::new("bash");
                command.arg(bash_args);
                command.env("COOKBOOK_REDOXER", "cargo");
                command
            } else {
                let cookbook_redoxer = Path::new("target/release/cookbook_redoxer")
                    .canonicalize()
                    .unwrap_or(PathBuf::from("/bin/false"));
                let mut command = Command::new(&cookbook_redoxer);
                command.arg("env").arg("bash").arg(bash_args);
                command.env("COOKBOOK_REDOXER", &cookbook_redoxer);
                command
            };
            command.current_dir(&cookbook_build);
            command.env("TARGET", package_target(name));
            command.env("COOKBOOK_BUILD", &cookbook_build);
            command.env("COOKBOOK_NAME", name.name());
            command.env("COOKBOOK_HOST_TARGET", redoxer::host_target());
            command.env("COOKBOOK_RECIPE", &cookbook_recipe);
            command.env("COOKBOOK_ROOT", &cookbook_root);
            command.env("COOKBOOK_STAGE", &cookbook_stage);
            command.env("COOKBOOK_SOURCE", &cookbook_source);
            command.env("COOKBOOK_SYSROOT", &cookbook_sysroot);
            if let Some(cookbook_toolchain) = &cookbook_toolchain {
                command.env("COOKBOOK_TOOLCHAIN", cookbook_toolchain);
            } else if name.is_host() {
                command.env("COOKBOOK_TOOLCHAIN", &cookbook_sysroot);
            }
            command.env("COOKBOOK_MAKE_JOBS", cli_jobs.to_string());
            if cli_verbose {
                command.env("COOKBOOK_VERBOSE", "1");
            }
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

        // Move to each features dir
        let mut globs = Vec::new();
        for (i, feat) in recipe.optional_packages.iter().enumerate() {
            let stage_dir = &stage_dirs[i];
            create_dir_clean(&stage_dir)?;
            for path in &feat.files {
                let glob = globset::Glob::new(&path).map_err(|e| format!("{}", e))?;
                globs.push((glob.compile_matcher(), stage_dir.clone()));
            }
        }
        move_dir_all_fn(
            &stage_dir_tmp,
            &Box::new(|path: PathBuf| {
                for (glob, dst) in &globs {
                    if glob.is_match(&path) {
                        return Some(dst.as_path());
                    }
                }
                None
            }),
        )
        .map_err(|e| format!("Unable to move {e:?}"))?;

        // Move stage.tmp to stage atomically
        rename(&stage_dir_tmp, &stage_dir)?;
    }

    let auto_deps = build_auto_deps(recipe, target_dir, &stage_dirs, dep_pkgars, logger)?;

    Ok((stage_dirs, auto_deps))
}

fn get_stage_dirs(features: &Vec<OptionalPackageRecipe>, target_dir: &Path) -> Vec<PathBuf> {
    let mut v = Vec::new();
    for f in features {
        v.push(target_dir.join(format!("stage.{}", f.name)));
    }
    // intentionally added last as it contains leftover files from package features
    v.push(target_dir.join(format!("stage")));
    v
}

fn build_deps_dir(
    logger: &PtyOut,
    deps_dir: &PathBuf,
    deps_dir_tmp: PathBuf,
    dep_pkgars: &BTreeSet<(PackageName, PathBuf)>,
    source_modified: SystemTime,
    deps_modified: SystemTime,
) -> Result<(), String> {
    if deps_dir.is_dir() {
        let tags_dir = deps_dir.join(".tags");
        let sysroot_modified = modified_dir(&tags_dir).unwrap_or(SystemTime::UNIX_EPOCH);
        if sysroot_modified < source_modified
            || sysroot_modified < deps_modified
            || dep_pkgars
                .iter()
                .any(|(pkg, _)| !tags_dir.join(pkg.as_str()).is_file())
        {
            log_to_pty!(logger, "DEBUG: updating '{}'", deps_dir.display());
            remove_all(deps_dir)?;
        }
    }
    if !deps_dir.is_dir() {
        // Create sysroot.tmp
        create_dir_clean(&deps_dir_tmp)?;
        let tags_dir = deps_dir_tmp.join(".tags");
        let usr_dir = deps_dir_tmp.join("usr");
        create_dir(&tags_dir)?;
        create_dir(&usr_dir)?;

        for folder in &["bin", "include", "lib", "share"] {
            // Make sure sysroot/usr/$folder exists
            create_dir(&usr_dir.join(folder))?;

            // Link sysroot/$folder sysroot/usr/$folder
            symlink(Path::new("usr").join(folder), &deps_dir_tmp.join(folder))?;
        }

        let pkey_path = "build/id_ed25519.pub.toml";
        for (name, archive_path) in dep_pkgars {
            let tag_file = tags_dir.join(name.without_host().as_str());
            fs::write(&tag_file, "")
                .map_err(|e| format!("failed to write tag file {}: {:?}", tag_file.display(), e))?;
            pkgar::extract(pkey_path, &archive_path, deps_dir_tmp.to_str().unwrap()).map_err(
                |err| {
                    format!(
                        "failed to install '{}' in '{}': {:?}",
                        archive_path.display(),
                        deps_dir_tmp.display(),
                        err
                    )
                },
            )?;
        }

        // Move sysroot.tmp to sysroot atomically
        rename(&deps_dir_tmp, deps_dir)?;
    }

    Ok(())
}

/// Calculate automatic dependencies
fn build_auto_deps(
    recipe: &Recipe,
    target_dir: &Path,
    stage_dirs: &Vec<PathBuf>,
    mut dep_pkgars: BTreeSet<(PackageName, PathBuf)>,
    logger: &PtyOut,
) -> Result<BTreeSet<PackageName>, String> {
    let auto_deps_path = target_dir.join("auto_deps.toml");
    if auto_deps_path.is_file() && modified(&auto_deps_path)? < modified_all(stage_dirs, modified)?
    {
        remove_all(&auto_deps_path)?
    }

    let auto_deps = if auto_deps_path.exists() {
        let toml_content =
            fs::read_to_string(&auto_deps_path).map_err(|_| "failed to read cached auto_deps")?;
        let wrapper: AutoDeps =
            toml::from_str(&toml_content).map_err(|_| "failed to deserialize cached auto_deps")?;
        wrapper.packages
    } else {
        let mut dynamic_deps =
            auto_deps_from_dynamic_linking(stage_dirs, target_dir, &dep_pkgars, logger);
        dep_pkgars.retain(|x| recipe.build.dependencies.contains(&x.0));
        let package_deps =
            auto_deps_from_static_package_deps(&dep_pkgars, &dynamic_deps).unwrap_or_default();
        dynamic_deps.extend(package_deps);

        let wrapper = AutoDeps {
            packages: dynamic_deps,
        };
        serialize_and_write(&auto_deps_path, &wrapper)?;
        wrapper.packages
    };
    Ok(auto_deps)
}

pub fn build_remote(
    stage_dirs: Vec<PathBuf>,
    recipe: &Recipe,
    target_dir: &Path,
) -> Result<(Vec<PathBuf>, BTreeSet<PackageName>), String> {
    let source_toml = target_dir.join("source.toml");
    let source_pubkey = target_dir.join("id_ed25519.pub.toml");

    let packages = recipe.get_packages_list();
    for (i, package) in packages.into_iter().enumerate() {
        // declare pkg dependencies as autodeps dependency
        let stage_dir = &stage_dirs[i];

        if !stage_dir.is_dir() {
            let (_, source_pkgar, _) = package_source_paths(package, &target_dir);
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
    }

    let auto_deps_path = target_dir.join("auto_deps.toml");
    if auto_deps_path.is_file() && modified(&auto_deps_path)? < modified_all(&stage_dirs, modified)?
    {
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
    Ok((stage_dirs, auto_deps))
}

#[cfg(test)]
mod tests {
    use std::os::unix;

    #[test]
    fn file_system_loop_no_infinite_loop() {
        let mut root = std::env::temp_dir();
        root.push("temp_test_dir_file_system_loop_no_infinite_loop");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("Failed to create temporary root directory");

        // Hierarchy with an infinite loop
        let dir = root.join("loop");
        unix::fs::symlink(&root, &dir).expect("Linking {dir:?} to {root:?}");

        // Sanity check that we have a loop
        assert_eq!(
            root.canonicalize().unwrap(),
            dir.canonicalize().unwrap(),
            "Expected a loop where {dir:?} points to {root:?}"
        );

        let entries = super::auto_deps_from_dynamic_linking(
            &root,
            &root.join(".."),
            &Default::default(),
            &None,
        );
        assert!(
            entries.is_empty(),
            "auto_deps shouldn't have yielded any libraries"
        );
    }
}
