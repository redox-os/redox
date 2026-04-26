use pkg::{Package, PackageError, PackageName};
use pkgar::{PackageFile, Transaction};
use pkgar_core::PackageSrc;
use pkgar_keys::PublicKeyFile;

use crate::config::CookConfig;
use crate::cook::package::{package_source_paths, package_target};
use crate::cook::{fetch, fs, pty::PtyOut, script::*};
use crate::recipe::{AutoDeps, BuildKind, CookRecipe, OptionalPackageRecipe, Recipe};
use std::{
    collections::{BTreeSet, VecDeque},
    path::{Path, PathBuf},
    process::Command,
    str,
};

use crate::{Error, Result, is_redox, log_to_pty, wrap_io_err};

fn auto_deps_from_dynamic_linking(
    stage_dirs: &[PathBuf],
    dep_pkgars: &BTreeSet<(PackageName, PathBuf)>,
    logger: &PtyOut,
) -> BTreeSet<PackageName> {
    let mut paths = BTreeSet::new();
    let mut visited = BTreeSet::new();
    let verbose = crate::config::get_config().cook.verbose;
    // Base directories may need to be updated for packages that place binaries in odd locations.
    let mut walk = VecDeque::new();

    for stage_dir in stage_dirs {
        walk.push_back((stage_dir, stage_dir.join("usr/bin")));
        walk.push_back((stage_dir, stage_dir.join("usr/games")));
        walk.push_back((stage_dir, stage_dir.join("usr/lib")));
        walk.push_back((stage_dir, stage_dir.join("usr/libexec")));
    }

    // Recursively (DFS) walk each directory to ensure nested libs and bins are checked.
    while let Some((rel_path, dir)) = walk.pop_front() {
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

        let Ok(read_dir) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry_res in read_dir {
            let Ok(entry) = entry_res else { continue };
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_file() {
                paths.insert((rel_path, entry.path()));
            } else if file_type.is_dir() {
                walk.push_front((rel_path, entry.path()));
            }
        }
    }

    let mut needed = BTreeSet::new();
    for (rel_path, path) in paths {
        let Ok(file) = std::fs::File::open(&path) else {
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
                if let Ok(relative_path) = path.strip_prefix(rel_path) {
                    if verbose {
                        log_to_pty!(logger, "DEBUG: {} needs {}", relative_path.display(), name);
                    }
                }
                needed.insert(name.to_string());
            } else {
                log_to_pty!(
                    logger,
                    "DEBUG: autopath failed {} is outside {}",
                    path.display(),
                    rel_path.display()
                );
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
                        deps.insert(dep.with_prefix(pkg::PackagePrefix::Any));
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
) -> std::result::Result<BTreeSet<PackageName>, PackageError> {
    let static_dep_pkgars: Vec<PackageName> = build_dep_pkgars
        .iter()
        .map(|x| x.0.clone())
        .filter(|x| !dynamic_dep_pkgars.contains(x))
        .collect();
    let pkgs = CookRecipe::get_package_deps_recursive(&static_dep_pkgars, false)?;

    Ok(pkgs.into_iter().collect())
}

pub struct BuildResult {
    pub stage_dirs: Vec<PathBuf>,
    pub auto_deps: BTreeSet<PackageName>,
    pub cached: bool,
}

impl BuildResult {
    pub fn new(stage_dirs: Vec<PathBuf>, auto_deps: BTreeSet<PackageName>) -> Self {
        BuildResult {
            stage_dirs,
            auto_deps,
            cached: false,
        }
    }

    pub fn cached(stage_dirs: Vec<PathBuf>, auto_deps: BTreeSet<PackageName>) -> Self {
        BuildResult {
            stage_dirs,
            auto_deps,
            cached: true,
        }
    }
}

pub fn build(
    recipe_dir: &Path,
    source_dir: &Path,
    target_dir: &Path,
    cook_recipe: &CookRecipe,
    cook_config: &CookConfig,
    logger: &PtyOut,
) -> Result<BuildResult> {
    let recipe = &cook_recipe.recipe;
    let name = &cook_recipe.name;
    let check_source = !cook_recipe.is_deps;
    let sysroot_dir = get_sub_target_dir(target_dir, "sysroot");
    let toolchain_dir = get_sub_target_dir(target_dir, "toolchain");
    let auto_deps_file = get_sub_target_dir(target_dir, "auto_deps.toml");
    let stage_dirs = get_stage_dirs(&recipe.optional_packages, target_dir);
    let stage_pkgars: Vec<PathBuf> = stage_dirs
        .iter()
        .map(|p| p.with_added_extension("pkgar"))
        .collect();
    let cli_jobs = cook_config.jobs;
    if recipe.build.kind == BuildKind::None {
        // metapackages don't need to do anything here
        return Ok(BuildResult::new(stage_dirs, BTreeSet::new()));
    }

    let mut dep_pkgars = BTreeSet::new();
    let mut dep_host_pkgars = BTreeSet::new();
    let build_deps = CookRecipe::get_build_deps_recursive(
        &[
            &recipe.build.dependencies[..],
            &recipe.build.dev_dependencies[..],
        ]
        .concat(),
        false,
    )?;
    for dependency in build_deps.iter() {
        let (_, pkgar, _) = dependency.stage_paths();
        if dependency.name.is_host() {
            dep_host_pkgars.insert((dependency.name.clone(), pkgar));
        } else {
            dep_pkgars.insert((dependency.name.clone(), pkgar));
        }
    }

    macro_rules! make_auto_deps {
        ($cached:expr) => {
            build_auto_deps(
                recipe,
                &auto_deps_file,
                &stage_dirs,
                $cached,
                cook_config,
                dep_pkgars,
                logger,
            )
        };
    }

    if !check_source {
        // TODO: when stage_dirs does not exist due to clean_target was true, extract from stage.pkgar?
        let stage_present = stage_pkgars.iter().all(|file| file.is_file());
        if stage_present && auto_deps_file.is_file() {
            log_to_pty!(logger, "DEBUG: using cached build, not checking source");
            let auto_deps = make_auto_deps!(true)?;
            return Ok(BuildResult::cached(stage_dirs, auto_deps));
        }
    }

    if recipe.build.kind == BuildKind::Remote {
        return build_remote(stage_dirs, stage_pkgars, recipe, target_dir, logger);
    }

    let deps_sysroot = if name.is_host() {
        &dep_host_pkgars
    } else {
        &dep_pkgars
    };
    let have_toolchain = !name.is_host() && dep_host_pkgars.len() > 0;
    let (sysroot_cached, toolchain_cached) = (
        build_deps_dir(logger, &sysroot_dir, deps_sysroot)?,
        if have_toolchain {
            build_deps_dir(logger, &toolchain_dir, &dep_host_pkgars)?
        } else {
            true
        },
    );

    // Rebuild stage if deps or source is newer
    if !sysroot_cached || !toolchain_cached || {
        build_is_source_newer(
            logger,
            recipe_dir,
            source_dir,
            &auto_deps_file,
            stage_pkgars,
        )
    } {
        if auto_deps_file.is_file() {
            fs::remove_all(&auto_deps_file)?;
        }
        for stage_dir in &stage_dirs {
            remove_stage_dir(stage_dir)?;
        }
        if cook_config.clean_target {
            // no matter what, these two caches are invalid
            if sysroot_cached {
                fs::remove_all(&sysroot_dir)?;
                build_deps_dir(logger, &sysroot_dir, deps_sysroot)?;
            }
            if toolchain_cached && have_toolchain {
                fs::remove_all(&toolchain_dir)?;
                build_deps_dir(logger, &toolchain_dir, &dep_host_pkgars)?;
            }
        }
    } else {
        log_to_pty!(logger, "DEBUG: using cached build");
        // stop early otherwise we'll end up rebuilding
        let auto_deps = make_auto_deps!(true)?;
        return Ok(BuildResult::cached(stage_dirs, auto_deps));
    }

    let stage_dir = stage_dirs
        .last()
        .expect("Should have atleast one stage dir");

    let build_dir = get_sub_target_dir(target_dir, "build");
    if !stage_dir.is_dir() {
        // Create stage.tmp
        let stage_dir_tmp = target_dir.join("stage.tmp");
        fs::create_dir_clean(&stage_dir_tmp)?;

        // Create build dir, if it does not exist
        if cook_config.clean_build || !build_dir.is_dir() {
            fs::create_dir_clean(&build_dir)?;
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

        let mut allow_cargo_offline = false;
        //TODO: better integration with redoxer (library instead of binary)
        //TODO: configurable target
        //TODO: Add more configurability, convert scripts to Rust?
        let script = match &recipe.build.kind {
            BuildKind::Cargo {
                cargopath,
                cargoflags,
                cargopackages,
                cargoexamples,
            } => {
                allow_cargo_offline = true;
                let mut script = format!(
                    "DYNAMIC_INIT\n{}\nCOOKBOOK_CARGO_PATH={} ",
                    flags_fn("COOKBOOK_CARGO_FLAGS", cargoflags),
                    cargopath.as_deref().unwrap_or(".")
                );
                if cargopackages.len() == 0 && cargoexamples.len() == 0 {
                    script += "cookbook_cargo\n"
                } else {
                    if cargopackages.len() > 0 {
                        script += "cookbook_cargo_packages";
                        for package in cargopackages {
                            script += " ";
                            script += package;
                        }
                        script += "\n";
                    }
                    if cargoexamples.len() > 0 {
                        script += "cookbook_cargo_examples";
                        for example in cargoexamples {
                            script += " ";
                            script += example;
                        }
                        script += "\n";
                    }
                }

                script
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
            let bash_args = if cook_config.verbose_cmd { "-ex" } else { "-e" };
            let local_redoxer = Path::new("target/release/cookbook_redoxer");
            let mut command = if is_redox() && !local_redoxer.is_file() {
                let mut command = Command::new("cookbook_redoxer");
                command.env("COOKBOOK_REDOXER", "cookbook_redoxer");
                command
            } else {
                let cookbook_redoxer = local_redoxer
                    .canonicalize()
                    .unwrap_or(PathBuf::from("/bin/false"));
                let mut command = Command::new(&cookbook_redoxer);
                command.env("COOKBOOK_REDOXER", &cookbook_redoxer);
                command
            };
            command.arg("env").arg("bash").arg(bash_args);
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
            if cook_config.verbose_cmd {
                command.env("COOKBOOK_VERBOSE", "1");
            }
            if cook_config.offline && allow_cargo_offline {
                command.env("COOKBOOK_OFFLINE", "1");
            } else {
                command.env_remove("COOKBOOK_OFFLINE");
            }
            if let Ok(ident_source) = fetch::fetch_get_source_info(&cook_recipe) {
                command.env("COOKBOOK_SOURCE_IDENT", ident_source.source_identifier);
                command.env("COOKBOOK_COMMIT_IDENT", ident_source.commit_identifier);
            }
            command
        };

        let full_script = format!(
            "{}\n{}\n{}\n{}",
            BUILD_PRESCRIPT, SHARED_PRESCRIPT, script, BUILD_POSTSCRIPT
        );
        fs::run_command_stdin(command, full_script.as_bytes(), logger)?;

        // Move to each features dir
        let mut globs = Vec::new();
        for (i, feat) in recipe.optional_packages.iter().enumerate() {
            let stage_dir = &stage_dirs[i];
            fs::create_dir_clean(&stage_dir)?;
            for path in &feat.files {
                let glob = globset::Glob::new(&path).map_err(|e| format!("{}", e))?;
                globs.push((glob.compile_matcher(), stage_dir.clone()));
            }
        }
        fs::move_dir_all_fn(
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
        .map_err(wrap_io_err!("Moving to stages dir"))?;

        // Move stage.tmp to stage atomically
        fs::rename(&stage_dir_tmp, &stage_dir)?;
    }

    if cook_config.clean_target {
        fs::remove_all(&build_dir)?;
        clean_deps_dir(&sysroot_dir)?;
        if toolchain_dir.is_dir() {
            clean_deps_dir(&toolchain_dir)?;
        }
        // don't remove stage dir yet
    }

    let auto_deps = make_auto_deps!(false)?;
    Ok(BuildResult::new(stage_dirs, auto_deps))
}

fn build_is_source_newer(
    logger: &PtyOut,
    recipe_dir: &Path,
    source_dir: &Path,
    auto_deps_file: &Path,
    stage_pkgars: Vec<PathBuf>,
) -> bool {
    if !auto_deps_file.is_file() {
        return true;
    }
    let Ok(stage_modified) = fs::modified_all(&stage_pkgars, fs::modified) else {
        return true;
    };
    let Ok(mut source_modified) = fs::modified_dir_ignore_git(source_dir) else {
        return false;
    };
    let mut recipe_is_newest = false;
    if let Ok(recipe_modified) = fs::modified(&recipe_dir.join("recipe.toml")) {
        if recipe_modified > source_modified {
            source_modified = recipe_modified;
            recipe_is_newest = true;
        }
    }
    let newer = source_modified > stage_modified;
    if newer {
        log_to_pty!(
            logger,
            "DEBUG: updating build: {} is newer",
            if recipe_is_newest { "recipe" } else { "source" }
        );
    }
    newer
}

pub fn remove_stage_dir(stage_dir: &PathBuf) -> crate::Result<()> {
    if stage_dir.is_dir() {
        fs::remove_all(&stage_dir)?;
    }
    let stage_file = stage_dir.with_added_extension("pkgar");
    if stage_file.is_file() {
        fs::remove_all(&stage_file)?;
    }
    let stage_meta = stage_dir.with_added_extension("toml");
    if stage_meta.is_file() {
        fs::remove_all(&stage_meta)?;
    }
    let stage_files = stage_dir.with_added_extension("files");
    if stage_files.is_file() {
        fs::remove_all(&stage_files)?;
    }
    Ok(())
}

pub fn get_stage_dirs(features: &Vec<OptionalPackageRecipe>, target_dir: &Path) -> Vec<PathBuf> {
    let mut target_dir = target_dir.to_path_buf();
    if let Some(cross_target) = crate::cross_target() {
        // TODO: automatically pass COOKBOOK_CROSS_GNU_TARGET?
        target_dir = target_dir.join(cross_target)
    }
    let mut v = Vec::new();
    for f in features {
        v.push(target_dir.join(format!("stage.{}", f.name)));
    }
    // intentionally added last as it contains leftover files from package features
    v.push(target_dir.join("stage"));
    v
}

pub fn get_sub_target_dir(target_dir: &Path, sub_path: &str) -> PathBuf {
    let mut target_dir = target_dir.to_path_buf();
    if let Some(cross_target) = crate::cross_target() {
        // TODO: automatically pass COOKBOOK_CROSS_GNU_TARGET?
        target_dir = target_dir.join(cross_target)
    }
    target_dir.join(sub_path)
}

fn build_deps_dir(
    logger: &PtyOut,
    deps_dir: &PathBuf,
    dep_pkgars: &BTreeSet<(PackageName, PathBuf)>,
) -> Result<bool> {
    let pkey_path = "build/id_ed25519.pub.toml";
    let pkey_file = match PublicKeyFile::open(&pkey_path) {
        Ok(k) => k.pkey,
        Err(e) => {
            if dep_pkgars.len() > 0 {
                return Err(Error::from(e));
            } else {
                // should never be accessed
                Default::default()
            }
        }
    };
    let tags_dir = deps_dir.join(".tags");
    if tags_dir.is_dir() {
        // check all files present and exact
        let mut cached = fs::check_files_present(
            &tags_dir,
            &dep_pkgars
                .iter()
                .map(|(name, _)| name.without_prefix())
                .collect(),
        )?;
        if cached {
            for (name, pkgar_path) in dep_pkgars {
                let tag_file = tags_dir.join(name.without_prefix());
                let Ok(tag_hash) = blake3::Hash::from_hex(fs::read_to_string(&tag_file)?) else {
                    log_to_pty!(
                        logger,
                        "DEBUG: updating {:?}: {:?} is absent",
                        deps_dir.file_name().unwrap().display(),
                        name.as_str(),
                    );
                    cached = false;
                    continue;
                };
                let pkgar_hash = PackageFile::new(pkgar_path, &pkey_file)?.header().blake3;
                if *tag_hash.as_bytes() != pkgar_hash {
                    log_to_pty!(
                        logger,
                        "DEBUG: updating {:?}: {:?} is updated",
                        deps_dir.file_name().unwrap().display(),
                        name.as_str(),
                    );
                    cached = false;
                }
            }
        }
        if cached {
            return Ok(true);
        }
        fs::remove_all(deps_dir)?;
    }

    // Create sysroot.tmp
    let deps_dir_tmp = deps_dir.with_added_extension("tmp");
    fs::create_dir_clean(&deps_dir_tmp)?;
    let tags_dir = deps_dir_tmp.join(".tags");
    let usr_dir = deps_dir_tmp.join("usr");
    fs::create_dir(&tags_dir)?;
    fs::create_dir(&usr_dir)?;

    for folder in &["bin", "include", "lib", "share"] {
        // Make sure sysroot/usr/$folder exists
        fs::create_dir(&usr_dir.join(folder))?;

        // Link sysroot/$folder sysroot/usr/$folder
        fs::symlink(Path::new("usr").join(folder), &deps_dir_tmp.join(folder))?;
    }

    for (name, archive_path) in dep_pkgars {
        let tag_file = tags_dir.join(name.without_prefix());
        let mut package = PackageFile::new(archive_path, &pkey_file)?;
        Transaction::install(&mut package, &deps_dir_tmp)?.commit()?;
        let hash = blake3::Hash::from_bytes(package.header().blake3).to_hex();
        std::fs::write(&tag_file, &hash.as_bytes()).map_err(wrap_io_err!("Writing tag"))?;
    }

    // Move sysroot.tmp to sysroot atomically
    fs::rename(&deps_dir_tmp, deps_dir)?;

    Ok(false)
}

fn clean_deps_dir(deps_dir: &PathBuf) -> Result<bool> {
    // this retain tags for future check
    let tags_dir = deps_dir.join(".tags");
    let tags_dir_tmp = deps_dir.with_added_extension("tags");
    fs::rename(&tags_dir, &tags_dir_tmp)?;
    fs::remove_all(&deps_dir)?;
    fs::create_dir(&deps_dir)?;
    fs::rename(&tags_dir_tmp, &tags_dir)?;
    Ok(false)
}

/// Calculate automatic dependencies
fn build_auto_deps(
    recipe: &Recipe,
    auto_deps_path: &Path,
    stage_dirs: &Vec<PathBuf>,
    cached: bool,
    cook_config: &CookConfig,
    mut dep_pkgars: BTreeSet<(PackageName, PathBuf)>,
    logger: &PtyOut,
) -> Result<BTreeSet<PackageName>> {
    if auto_deps_path.is_file() && !cached {
        if cook_config.verbose {
            log_to_pty!(logger, "DEBUG: updating {}", auto_deps_path.display());
        }
        fs::remove_all(&auto_deps_path)?;
    }

    let auto_deps = if auto_deps_path.exists() {
        let wrapper: AutoDeps = fs::read_toml(&auto_deps_path)?;
        wrapper.packages
    } else {
        let mut dynamic_deps = auto_deps_from_dynamic_linking(stage_dirs, &dep_pkgars, logger);
        dep_pkgars.retain(|x| recipe.build.dependencies.contains(&x.0));
        let package_deps =
            auto_deps_from_static_package_deps(&dep_pkgars, &dynamic_deps).unwrap_or_default();
        dynamic_deps.extend(package_deps);

        let wrapper = AutoDeps {
            packages: dynamic_deps,
        };
        fs::serialize_and_write(&auto_deps_path, &wrapper)?;
        wrapper.packages
    };
    Ok(auto_deps)
}

pub fn build_remote(
    stage_dirs: Vec<PathBuf>,
    stage_pkgars: Vec<PathBuf>,
    recipe: &Recipe,
    target_dir: &Path,
    logger: &PtyOut,
) -> Result<BuildResult> {
    let source_toml = target_dir.join("source.toml");
    let source_pubkey = "build/remotes/pub_key_static.redox-os.org.toml";
    let auto_deps_path = target_dir.join("auto_deps.toml");

    let packages = recipe.get_packages_list();
    let mut cached = auto_deps_path.is_file();
    for (i, package) in packages.iter().enumerate() {
        let stage_pkgar = &stage_pkgars[i];

        if !stage_pkgar.is_file() {
            cached = false;
            break;
        }

        let (_, source_pkgar, _) = package_source_paths(*package, &target_dir);
        if fs::modified(&source_pkgar)? > fs::modified(&stage_pkgar)? {
            cached = false;
            break;
        }
    }

    if cached {
        log_to_pty!(logger, "DEBUG: using cached build");
        let wrapper: AutoDeps = fs::read_toml(&auto_deps_path)?;
        return Ok(BuildResult::cached(stage_dirs, wrapper.packages));
    }

    for (i, package) in packages.into_iter().enumerate() {
        let stage_dir = &stage_dirs[i];
        let (_, source_pkgar, _) = package_source_paths(package, &target_dir);
        fs::create_dir_clean(stage_dir)?;
        pkgar::extract(&source_pubkey, &source_pkgar, &stage_dir)?;
    }

    if auto_deps_path.is_file() {
        fs::remove_all(&auto_deps_path)?
    }

    let auto_deps = {
        let toml_content = fs::read_to_string(&source_toml)?;
        let pkg_toml = Package::from_toml(&toml_content)?;
        let wrapper = AutoDeps {
            packages: pkg_toml.depends.into_iter().collect(),
        };
        fs::serialize_and_write(&auto_deps_path, &wrapper)?;
        wrapper.packages
    };
    Ok(BuildResult::new(stage_dirs, auto_deps))
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

        let entries =
            super::auto_deps_from_dynamic_linking(&vec![root.clone()], &Default::default(), &None);
        assert!(
            entries.is_empty(),
            "auto_deps shouldn't have yielded any libraries"
        );
    }
}
