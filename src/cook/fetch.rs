use crate::Error;
use crate::Result;
use crate::bail_other_err;
use crate::config::translate_mirror;
use crate::cook::cook_build;
use crate::cook::fetch_repo;
use crate::cook::fetch_repo::PlainPtyCallback;
use crate::cook::fs::*;
use crate::cook::package::get_package_name;
use crate::cook::package::package_source_paths;
use crate::cook::pty::PtyOut;
use crate::cook::script::*;
use crate::is_redox;
use crate::log_to_pty;
use crate::recipe::BuildKind;
use crate::recipe::CookRecipe;
use crate::recipe::SourceRecipe;
use crate::wrap_io_err;
use crate::wrap_other_err;
use pkg::SourceIdentifier;
use pkg::net_backend::DownloadBackendWriter;
use std::cell::RefCell;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;

pub struct FetchResult {
    pub source_dir: PathBuf,
    pub source_ident: String,
    pub cached: bool,
}

impl FetchResult {
    pub fn new(source_dir: PathBuf, ident: String, cached: bool) -> Self {
        Self {
            source_dir,
            source_ident: ident,
            cached,
        }
    }

    pub fn cached(source_dir: PathBuf, ident: String) -> Self {
        Self {
            source_dir,
            source_ident: ident,
            cached: true,
        }
    }
}

pub(crate) fn get_blake3(path: &PathBuf) -> Result<String> {
    let mut f = fs::File::open(&path).map_err(wrap_io_err!(path, "Opening file for blake3"))?;
    let hash = blake3::Hasher::new()
        .update_reader(&mut f)
        .map_err(wrap_io_err!(path, "Reading file for blake3"))?
        .finalize();
    Ok(hash.to_hex().to_string())
}

pub fn fetch_offline(recipe: &CookRecipe, logger: &PtyOut) -> Result<FetchResult> {
    let recipe_dir = &recipe.dir;
    let source_dir = recipe_dir.join("source");
    match recipe.recipe.build.kind {
        BuildKind::None => {
            // the build function doesn't need source dir exists
            let ident = fetch_apply_source_info(recipe, "".to_string())?;
            return Ok(FetchResult::cached(source_dir, ident));
        }
        BuildKind::Remote => {
            return fetch_remote(recipe_dir, recipe, true, source_dir, logger);
        }
        _ => {}
    }

    let result = match &recipe.recipe.source {
        Some(SourceRecipe::Path { path: _ }) | None => fetch(recipe, true, logger)?,
        Some(SourceRecipe::SameAs { same_as }) => {
            let recipe = fetch_resolve_canon(recipe_dir, &same_as, recipe.name.is_host())?;
            // recursively fetch
            let r = fetch_offline(&recipe, logger)?;
            fetch_make_symlink(&source_dir, &same_as)?;
            r
        }
        Some(SourceRecipe::Git {
            git: _,
            upstream: _,
            branch: _,
            rev: _,
            patches: _,
            script: _,
            shallow_clone: _,
        }) => {
            offline_check_exists(&source_dir)?;
            let (head_rev, _) = get_git_head_rev(&source_dir)?;
            FetchResult::cached(source_dir, head_rev)
        }
        Some(SourceRecipe::Tar {
            tar: _,
            blake3,
            patches,
            script,
        }) => {
            let ident = blake3.clone().unwrap_or("no_tar_blake3_hash_info".into());
            let cached = source_dir.is_dir();
            if !cached {
                let source_tar = recipe_dir.join("source.tar");
                let source_tar_blake3 = get_blake3(&source_tar)?;
                if source_tar.exists() {
                    if let Some(blake3) = blake3 {
                        if source_tar_blake3 != *blake3 {
                            bail_other_err!(
                                "The downloaded tar blake3 {source_tar_blake3:?} is not equal to blake3 in recipe.toml"
                            );
                        }
                        create_dir(&source_dir)?;
                        fetch_extract_tar(source_tar, &source_dir, logger)?;
                        fetch_apply_patches(recipe_dir, patches, script, &source_dir, logger)?;
                    } else {
                        // need to trust this tar file
                        bail_other_err!(
                            "Please add blake3 = {source_tar_blake3:?} to {recipe:?}",
                            recipe = recipe_dir.join("recipe.toml").display(),
                        );
                    }
                }
            }
            offline_check_exists(&source_dir)?;
            FetchResult::new(source_dir, ident, cached)
        }
    };

    fetch_apply_source_info(recipe, result.source_ident.clone())?;

    Ok(result)
}

pub fn fetch(recipe: &CookRecipe, check_source: bool, logger: &PtyOut) -> Result<FetchResult> {
    let recipe_dir = &recipe.dir;
    let source_dir = recipe_dir.join("source");
    match recipe.recipe.build.kind {
        BuildKind::None => {
            // the build function doesn't need source dir exists
            let ident = fetch_apply_source_info(recipe, "".to_string())?;
            return Ok(FetchResult::cached(source_dir, ident));
        }
        BuildKind::Remote => {
            return fetch_remote(recipe_dir, recipe, false, source_dir, logger);
        }
        _ => {}
    }

    let result = match &recipe.recipe.source {
        Some(SourceRecipe::SameAs { same_as }) => {
            let recipe = fetch_resolve_canon(recipe_dir, &same_as, recipe.name.is_host())?;
            // recursively fetch
            let r = fetch(&recipe, check_source, logger)?;
            fetch_make_symlink(&source_dir, &same_as)?;
            r
        }
        Some(SourceRecipe::Path { path }) => {
            let path = Path::new(&path);
            let cached = source_dir.is_dir() && modified_dir(path)? <= modified_dir(&source_dir)?;
            if !cached {
                log_to_pty!(
                    logger,
                    "[DEBUG]: {:?} is newer than {:?}",
                    path.display(),
                    source_dir.display()
                );
                copy_dir_all(path, &source_dir).map_err(wrap_io_err!(
                    path,
                    source_dir,
                    "Copying source"
                ))?;
            }
            FetchResult::new(source_dir, "local_source".to_string(), cached)
        }
        Some(SourceRecipe::Git {
            git,
            upstream,
            branch,
            rev,
            patches,
            script,
            shallow_clone,
        }) => {
            //TODO: use libgit?
            let shallow_clone = *shallow_clone == Some(true);
            let cached = if !source_dir.is_dir() {
                // Create source.tmp
                let source_dir_tmp = recipe_dir.join("source.tmp");
                create_dir_clean(&source_dir_tmp)?;

                // Clone the repository to source.tmp
                let mut command = Command::new("git");
                command
                    .arg("clone")
                    .arg("--recursive")
                    .arg(translate_mirror(&git));
                if let Some(branch) = branch {
                    command.arg("--branch").arg(branch);
                }
                if shallow_clone {
                    command
                        .arg("--filter=tree:0")
                        .arg("--also-filter-submodules");
                }
                command.arg(&source_dir_tmp);
                run_command(command, logger)?;

                // Move source.tmp to source atomically
                rename(&source_dir_tmp, &source_dir)?;

                false
            } else if !check_source {
                true
            } else {
                if !source_dir.join(".git").is_dir() {
                    bail_other_err!(
                        "{:?} is not a git repository, but recipe indicated git source",
                        source_dir.display()
                    );
                }

                // Reset origin
                let mut command = Command::new("git");
                command.arg("-C").arg(&source_dir);
                command.arg("remote").arg("set-url").arg("origin").arg(git);
                run_command(command, logger)?;

                // Fetch origin
                let mut command = Command::new("git");
                command.arg("-C").arg(&source_dir);
                command.arg("fetch").arg("origin");
                run_command(command, logger)?;

                let (head_rev, detached_rev) = get_git_head_rev(&source_dir)?;
                match (rev, detached_rev) {
                    (Some(rev), true) => {
                        if let Ok(exp_rev) = get_git_tag_rev(&source_dir, &rev) {
                            exp_rev == head_rev
                        } else {
                            let mut command = Command::new("git");
                            command.arg("-C").arg(&source_dir);
                            command.arg("gc");
                            run_command(command, logger)?;
                            if let Ok(exp_rev) = get_git_tag_rev(&source_dir, &rev) {
                                exp_rev == head_rev
                            } else {
                                false
                            }
                        }
                    }
                    (None, false) => {
                        let (_, remote_branch, remote_name, remote_url) =
                            get_git_remote_tracking(&source_dir)?;
                        // TODO: how to get default branch and compare it here?
                        if let Some(branch) = branch
                            && branch != &remote_branch
                        {
                            false
                        } else if remote_name != "origin" || &remote_url != chop_dot_git(git) {
                            false
                        } else {
                            match get_git_fetch_rev(&source_dir, &remote_url, &remote_branch) {
                                Ok(fetch_rev) => fetch_rev == head_rev,
                                Err(e) => {
                                    log_to_pty!(logger, "{}", e);
                                    false
                                }
                            }
                        }
                    }
                    _ => false,
                }
            };

            if !cached {
                if let Some(_upstream) = upstream {
                    //TODO: set upstream URL (is this needed?)
                    // git remote set-url upstream "$GIT_UPSTREAM" &> /dev/null ||
                    // git remote add upstream "$GIT_UPSTREAM"
                    // git fetch upstream
                }

                if !patches.is_empty() || script.is_some() {
                    // Hard reset
                    let mut command = Command::new("git");
                    command.arg("-C").arg(&source_dir);
                    command.arg("reset").arg("--hard");
                    run_command(command, logger)?;
                }

                if let Some(rev) = rev {
                    // Check out specified revision
                    let mut command = Command::new("git");
                    command.arg("-C").arg(&source_dir);
                    command.arg("checkout").arg(rev);
                    run_command(command, logger)?;
                } else if !is_redox() {
                    //TODO: complicated stuff to check and reset branch to origin
                    //TODO: redox can't undestand this (got exit status 1)
                    let mut command = Command::new("bash");
                    command.arg("-c").arg(GIT_RESET_BRANCH);
                    if let Some(branch) = branch {
                        command.env("BRANCH", branch);
                    }
                    command.current_dir(&source_dir);
                    run_command(command, logger)?;
                }

                // Sync submodules URL
                let mut command = Command::new("git");
                command.arg("-C").arg(&source_dir);
                command.arg("submodule").arg("sync").arg("--recursive");

                run_command(command, logger)?;

                // Update submodules
                let mut command = Command::new("git");
                command.arg("-C").arg(&source_dir);
                command
                    .arg("submodule")
                    .arg("update")
                    .arg("--init")
                    .arg("--recursive");
                if shallow_clone {
                    command.arg("--filter=tree:0");
                }
                run_command(command, logger)?;

                fetch_apply_patches(recipe_dir, patches, script, &source_dir, logger)?;
            }

            let (head_rev, _) = get_git_head_rev(&source_dir)?;
            FetchResult::new(source_dir, head_rev, cached)
        }
        Some(SourceRecipe::Tar {
            tar,
            blake3,
            patches,
            script,
        }) => {
            let source_tar = recipe_dir.join("source.tar");
            let ident = blake3.clone().unwrap_or("no_tar_blake3_hash_info".into());
            let mut tar_updated = false;
            loop {
                if !source_tar.is_file() {
                    tar_updated = true;
                    download_wget(&tar, &source_tar, logger)?;
                }
                if !check_source {
                    break;
                }
                let source_tar_blake3 = get_blake3(&source_tar)?;
                if let Some(blake3) = blake3 {
                    if source_tar_blake3 == *blake3 {
                        break;
                    }
                    if tar_updated {
                        bail_other_err!(
                            "The downloaded tar blake3 {source_tar_blake3:?} is not equal to blake3 in recipe.toml"
                        )
                    } else {
                        log_to_pty!(
                            logger,
                            "DEBUG: source tar blake3 is different and need redownload"
                        );
                        remove_all(&source_tar)?;
                    }
                } else {
                    //TODO: set blake3 hash on the recipe with something like "cook fix"
                    log_to_pty!(
                        logger,
                        "WARNING: set blake3 for '{}' to '{}'",
                        source_tar.display(),
                        source_tar_blake3
                    );
                    break;
                }
            }
            let mut cached = true;
            if source_dir.is_dir() {
                if tar_updated || fetch_is_patches_newer(recipe_dir, patches, &source_dir)? {
                    log_to_pty!(
                        logger,
                        "DEBUG: source tar or patches is newer than the source directory"
                    );
                    remove_all(&source_dir)?
                }
            }
            if !source_dir.is_dir() {
                // Create source.tmp
                let source_dir_tmp = recipe_dir.join("source.tmp");
                create_dir_clean(&source_dir_tmp)?;
                fetch_extract_tar(source_tar, &source_dir_tmp, logger)?;
                fetch_apply_patches(recipe_dir, patches, script, &source_dir_tmp, logger)?;

                // Move source.tmp to source atomically
                rename(&source_dir_tmp, &source_dir)?;
                cached = false;
            }
            FetchResult::new(source_dir, ident, cached)
        }
        // Local Sources
        None => {
            if !source_dir.is_dir() {
                log_to_pty!(
                    logger,
                    "WARNING: Recipe without source section expected source dir at '{}'",
                    source_dir.display(),
                );
                create_dir(&source_dir)?;
            }
            FetchResult::cached(source_dir, "local_source".into())
        }
    };

    if let BuildKind::Cargo {
        cargopath,
        cargoflags: _,
        cargopackages: _,
        cargoexamples: _,
    } = &recipe.recipe.build.kind
    {
        if fetch_will_build(recipe) {
            fetch_cargo(&result.source_dir, cargopath.as_ref(), logger)?;
        }
    }

    fetch_apply_source_info(recipe, result.source_ident.to_string())?;

    Ok(result)
}

/// This does the same check as in cook_build
fn fetch_will_build(recipe: &CookRecipe) -> bool {
    let check_source = !recipe.is_deps;
    if !check_source {
        // there could be more check here, but it's heavy so just assume it will build
        return true;
    }

    let stage_dirs =
        cook_build::get_stage_dirs(&recipe.recipe.optional_packages, &recipe.target_dir());
    let stage_pkgars: Vec<PathBuf> = stage_dirs
        .iter()
        .map(|p| p.with_added_extension("pkgar"))
        .collect();
    let stage_present = stage_pkgars.iter().all(|file| file.is_file());
    !stage_present
}

pub(crate) fn fetch_make_symlink(source_dir: &PathBuf, same_as: &String) -> Result<()> {
    let target_dir = Path::new(same_as).join("source");
    if !source_dir.is_symlink() {
        if source_dir.is_dir() {
            bail_other_err!(
                "'{dir:?}' is a directory, but recipe indicated a symlink. \n\
                        try removing '{dir:?}' if you haven't made any changes that would be lost",
                dir = source_dir.display(),
            )
        }
        std::os::unix::fs::symlink(&target_dir, source_dir).map_err(|err| {
            format!(
                "failed to symlink '{}' to '{}': {}\n{:?}",
                target_dir.display(),
                source_dir.display(),
                err,
                err
            )
        })?;
    }
    Ok(())
}

pub(crate) fn fetch_resolve_canon(
    recipe_dir: &Path,
    same_as: &String,
    is_host: bool,
) -> Result<CookRecipe> {
    let canon_dir = Path::new(recipe_dir).join(same_as);
    if canon_dir
        .to_str()
        .unwrap()
        .chars()
        .filter(|c| *c == '/')
        .count()
        > 50
    {
        bail_other_err!("Infinite loop detected");
    }
    if !canon_dir.exists() {
        bail_other_err!("{dir:?} is not exists", dir = canon_dir.display());
    }
    CookRecipe::from_path(canon_dir.as_path(), true, is_host).map_err(Error::from)
}

pub(crate) fn fetch_extract_tar(
    source_tar: PathBuf,
    source_dir_tmp: &PathBuf,
    logger: &PtyOut,
) -> Result<()> {
    let mut command = Command::new("tar");
    let verbose = crate::config::get_config().cook.verbose;
    if is_redox() {
        command.arg(if verbose { "xvf" } else { "xf" });
    } else {
        command.arg("--extract");
        command.arg("--no-same-owner");
        if verbose {
            command.arg("--verbose");
        }
        command.arg("--file");
    }
    command.arg(&source_tar);
    command.arg("--directory").arg(source_dir_tmp);
    command.arg("--strip-components").arg("1");
    run_command(command, logger)?;
    Ok(())
}

pub(crate) fn fetch_cargo(
    source_dir: &PathBuf,
    cargopath: Option<&String>,
    logger: &PtyOut,
) -> Result<()> {
    let mut source_dir = source_dir.clone();
    if let Some(cargopath) = cargopath {
        source_dir = source_dir.join(cargopath);
    }

    let local_redoxer = Path::new("target/release/cookbook_redoxer");
    let mut command = if is_redox() && !local_redoxer.is_file() {
        Command::new("cookbook_redoxer")
    } else {
        let cookbook_redoxer = local_redoxer
            .canonicalize()
            .unwrap_or(PathBuf::from("cargo"));
        Command::new(&cookbook_redoxer)
    };
    command.arg("fetch");
    command.arg("--manifest-path");
    command.arg(source_dir.join("Cargo.toml").into_os_string());
    run_command(command, logger)?;
    Ok(())
}

pub fn fetch_remote(
    recipe_dir: &Path,
    recipe: &CookRecipe,
    offline_mode: bool,
    source_dir: PathBuf,
    logger: &PtyOut,
) -> Result<FetchResult> {
    let (mut manager, repository) = fetch_repo::get_binary_repo();
    let target_dir = create_target_dir(recipe_dir, recipe.target)?;
    if logger.is_some() {
        let writer = logger.as_ref().unwrap().1.try_clone().unwrap();
        manager.set_callback(Rc::new(RefCell::new(PlainPtyCallback::new(writer))));
    }
    let packages = recipe.recipe.get_packages_list();

    let name = recipe_dir
        .file_name()
        .ok_or("Unable to get recipe name")?
        .to_str()
        .unwrap();

    let mut result = None;
    let mut cached = true;

    for package in packages {
        let (_, source_pkgar, source_toml) = package_source_paths(package, &target_dir);
        let source_name = get_package_name(name, package);
        let Some(repo_blake3) = repository.packages.get(&source_name) else {
            bail_other_err!("Package {source_name} does not exist in server repository")
        };

        if !offline_mode {
            if source_toml.is_file() {
                let pkg_toml = read_source_toml(&source_toml)?;
                if &pkg_toml.blake3 != repo_blake3 {
                    log_to_pty!(logger, "DEBUG: Updating source binaries");
                    remove_all(&source_toml)?;
                    if source_pkgar.is_file() {
                        remove_all(&source_pkgar)?;
                    }
                }
            }

            if !source_toml.is_file() {
                {
                    let toml_file = File::create(&source_toml)
                        .map_err(|e| format!("Unable to create source.toml: {e:?}"))?;
                    let mut writer = DownloadBackendWriter::ToFile(toml_file);
                    manager
                        .download(&format!("{}.toml", &source_name), None, &mut writer)
                        .map_err(|e| format!("Unable to download source.toml: {e:?}"))?;
                }
                let pkg_toml = read_source_toml(&source_toml)?;
                let pkgar_file = File::create(&source_pkgar)
                    .map_err(|e| format!("Unable to create source.pkgar: {e:?}"))?;
                let mut writer = DownloadBackendWriter::ToFile(pkgar_file);
                manager
                    .download(
                        &format!("{}.pkgar", &source_name),
                        Some(pkg_toml.network_size),
                        &mut writer,
                    )
                    .map_err(|e| format!("Unable to download source.pkgar: {e:?}"))?;

                cached = false;
            }

            // manager.download(file, 0, dest)
        } else {
            offline_check_exists(&source_pkgar)?;
            offline_check_exists(&source_toml)?;
        }

        // guaranteed to exist once and last in iteration
        if package.is_none() {
            let pkg_toml = read_source_toml(&source_toml)?;

            fetch_apply_source_info_from_remote(
                recipe,
                &SourceIdentifier {
                    commit_identifier: pkg_toml.commit_identifier.clone(),
                    source_identifier: pkg_toml.source_identifier.clone(),
                    time_identifier: pkg_toml.time_identifier.clone(),
                    ..Default::default()
                },
            )?;

            result = Some(FetchResult::new(
                source_dir.clone(),
                pkg_toml.source_identifier,
                cached,
            ));
        }
    }

    result.ok_or_else(wrap_other_err!("There's no mandatory package in remote"))
}

fn read_source_toml(source_toml: &Path) -> Result<pkg::Package> {
    let mut file =
        File::open(source_toml).map_err(|e| format!("Unable to open source.toml: {e:?}"))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| format!("Unable to read source.toml: {e:?}"))?;
    let pkg_toml = pkg::Package::from_toml(&contents)
        .map_err(|e| format!("Unable to parse source.toml: {e:?}"))?;
    Ok(pkg_toml)
}

pub(crate) fn fetch_is_patches_newer(
    recipe_dir: &Path,
    patches: &Vec<String>,
    source_dir: &PathBuf,
) -> Result<bool> {
    // don't check source files inside as it can be mixed with user patches
    let source_time = modified(&source_dir)?;
    for patch_name in patches {
        let patch_file = recipe_dir.join(patch_name);
        if !patch_file.is_file() {
            bail_other_err!("Failed to find patch file {:?}", patch_file.display());
        }

        let patch_time = modified(&patch_file)?;
        if patch_time > source_time {
            return Ok(true);
        }
    }
    return Ok(false);
}

pub(crate) fn fetch_apply_patches(
    recipe_dir: &Path,
    patches: &Vec<String>,
    script: &Option<String>,
    source_dir_tmp: &PathBuf,
    logger: &PtyOut,
) -> Result<()> {
    for patch_name in patches {
        let patch_file = recipe_dir.join(patch_name);
        if !patch_file.is_file() {
            bail_other_err!("Failed to find patch file {:?}", patch_file.display());
        }

        let patch = fs::read_to_string(&patch_file).map_err(|err| {
            format!(
                "failed to read patch file '{}': {}\n{:#?}",
                patch_file.display(),
                err,
                err
            )
        })?;

        let mut command = Command::new("patch");
        command.arg("--directory").arg(source_dir_tmp);
        command.arg("--strip=1");
        run_command_stdin(command, patch.as_bytes(), logger)?;
    }
    Ok(if let Some(script) = script {
        let mut command = Command::new("bash");
        command.arg("-ex");
        command.current_dir(source_dir_tmp);
        run_command_stdin(
            command,
            format!("{SHARED_PRESCRIPT}\n{script}").as_bytes(),
            logger,
        )?;
    })
}

pub(crate) fn fetch_apply_source_info(
    recipe: &CookRecipe,
    source_identifier: String,
) -> Result<String> {
    let ident = crate::cook::ident::get_ident();
    let info = SourceIdentifier {
        commit_identifier: ident.commit.to_string(),
        time_identifier: ident.time.to_string(),
        source_identifier: source_identifier,
    };

    fetch_apply_source_info_from_remote(&recipe, &info)?;

    Ok(info.source_identifier)
}

pub(crate) fn fetch_apply_source_info_from_remote(
    recipe: &CookRecipe,
    info: &SourceIdentifier,
) -> Result<()> {
    let target_dir = create_target_dir(&recipe.dir, recipe.target)?;
    let source_toml_path = target_dir.join("source_info.toml");
    serialize_and_write(&source_toml_path, &info)?;
    Ok(())
}

pub fn fetch_get_source_info(recipe: &CookRecipe) -> Result<SourceIdentifier> {
    let target_dir = recipe.target_dir();
    let source_toml_path = target_dir.join("source_info.toml");
    let toml_content = fs::read_to_string(source_toml_path)
        .map_err(|e| format!("Unable to read source_info.toml: {:?}", e))?;
    let parsed = toml::from_str(&toml_content)
        .map_err(|e| format!("Unable to parse source_info.toml: {:?}", e))?;
    Ok(parsed)
}
