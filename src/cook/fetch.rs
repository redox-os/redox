use pkg::package::SourceIdentifier;

use crate::REMOTE_PKG_SOURCE;
use crate::config::translate_mirror;
use crate::cook::fs::*;
use crate::cook::package::get_package_name;
use crate::cook::package::package_source_paths;
use crate::cook::pty::PtyOut;
use crate::cook::script::*;
use crate::is_redox;
use crate::log_to_pty;
use crate::recipe::BuildKind;
use crate::recipe::CookRecipe;
use crate::{blake3, recipe::SourceRecipe};
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(crate) fn get_blake3(path: &PathBuf, show_progress: bool) -> Result<String, String> {
    if show_progress {
        blake3::blake3_progress(&path)
    } else {
        blake3::blake3_silent(&path)
    }
    .map_err(|err| {
        format!(
            "failed to calculate blake3 of '{}': {}\n{:?}",
            path.display(),
            err,
            err
        )
    })
}

pub fn fetch_offline(recipe: &CookRecipe, logger: &PtyOut) -> Result<PathBuf, String> {
    let recipe_dir = &recipe.dir;
    let source_dir = recipe_dir.join("source");
    match recipe.recipe.build.kind {
        BuildKind::None => {
            // the build function doesn't need source dir exists
            fetch_apply_source_info(recipe, "".to_string())?;
            return Ok(source_dir);
        }
        BuildKind::Remote => {
            fetch_remote(recipe_dir, recipe, true, logger)?;
            return Ok(source_dir);
        }
        _ => {}
    }

    let ident = match &recipe.recipe.source {
        Some(SourceRecipe::Path { path: _ }) | None => {
            fetch(recipe, true, logger)?;
            "local_source".to_string()
        }
        Some(SourceRecipe::SameAs { same_as }) => {
            let recipe = fetch_resolve_canon(recipe_dir, &same_as, recipe.name.is_host())?;
            // recursively fetch
            fetch_offline(&recipe, logger)?;
            fetch_make_symlink(&source_dir, &same_as)?;
            fetch_get_source_info(&recipe)?.source_identifier
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
            head_rev
        }
        Some(SourceRecipe::Tar {
            tar: _,
            blake3,
            patches,
            script,
        }) => {
            if !source_dir.is_dir() {
                let source_tar = recipe_dir.join("source.tar");
                let source_tar_blake3 = get_blake3(&source_tar, true && logger.is_none())?;
                if source_tar.exists() {
                    if let Some(blake3) = blake3 {
                        if source_tar_blake3 != *blake3 {
                            return Err(format!(
                                "The downloaded tar blake3 '{source_tar_blake3}' is not equal to blake3 in recipe.toml."
                            ));
                        }
                        create_dir(&source_dir)?;
                        fetch_extract_tar(source_tar, &source_dir, logger)?;
                        fetch_apply_patches(recipe_dir, patches, script, &source_dir, logger)?;
                    } else {
                        // need to trust this tar file
                        return Err(format!(
                            "Please add blake3 = \"{source_tar_blake3}\" to '{recipe}'",
                            recipe = recipe_dir.join("recipe.toml").display(),
                        ));
                    }
                } else {
                    offline_check_exists(&source_dir)?;
                }
            }
            blake3.clone().unwrap_or("no_tar_blake3_hash_info".into())
        }
    };

    fetch_apply_source_info(recipe, ident)?;

    Ok(source_dir)
}

pub fn fetch(recipe: &CookRecipe, check_source: bool, logger: &PtyOut) -> Result<PathBuf, String> {
    let recipe_dir = &recipe.dir;
    let source_dir = recipe_dir.join("source");
    match recipe.recipe.build.kind {
        BuildKind::None => {
            // the build function doesn't need source dir exists
            fetch_apply_source_info(recipe, "".to_string())?;
            return Ok(source_dir);
        }
        BuildKind::Remote => {
            fetch_remote(recipe_dir, recipe, false, logger)?;
            return Ok(source_dir);
        }
        _ => {}
    }

    let ident = match &recipe.recipe.source {
        Some(SourceRecipe::SameAs { same_as }) => {
            let recipe = fetch_resolve_canon(recipe_dir, &same_as, recipe.name.is_host())?;
            // recursively fetch
            fetch(&recipe, check_source, logger)?;
            fetch_make_symlink(&source_dir, &same_as)?;
            fetch_get_source_info(&recipe)?.source_identifier
        }
        Some(SourceRecipe::Path { path }) => {
            if !source_dir.is_dir() || modified_dir(Path::new(&path))? > modified_dir(&source_dir)?
            {
                log_to_pty!(
                    logger,
                    "[DEBUG]: {} is newer than {}",
                    path,
                    source_dir.display()
                );
                copy_dir_all(path, &source_dir).map_err(|e| {
                    format!(
                        "Couldn't copy source from {} to {}: {}",
                        path,
                        source_dir.display(),
                        e
                    )
                })?;
            }
            "local_source".to_string()
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
            let can_skip_rebuild = if !source_dir.is_dir() {
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
                    return Err(format!(
                        "'{}' is not a git repository, but recipe indicated git source",
                        source_dir.display(),
                    ));
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
                if detached_rev {
                    if let Some(rev) = rev
                        && let Ok(exp_rev) = get_git_tag_rev(&source_dir, &rev)
                    {
                        exp_rev == head_rev
                    } else {
                        false
                    }
                } else {
                    let (_, remote_branch, remote_name, remote_url) =
                        get_git_remote_tracking(&source_dir)?;
                    // TODO: how to get default branch and compare it here?
                    if let Some(branch) = branch
                        && branch != &remote_branch
                    {
                        false
                    } else if remote_name != "origin" {
                        false
                    } else if &remote_url != chop_dot_git(git) {
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
            };

            if !can_skip_rebuild {
                if let Some(_upstream) = upstream {
                    //TODO: set upstream URL (is this needed?)
                    // git remote set-url upstream "$GIT_UPSTREAM" &> /dev/null ||
                    // git remote add upstream "$GIT_UPSTREAM"
                    // git fetch upstream
                }

                if let Some(rev) = rev {
                    // Check out specified revision
                    let mut command = Command::new("git");
                    command.arg("-C").arg(&source_dir);
                    command.arg("checkout").arg(rev);
                    run_command(command, logger)?;
                } else if !is_redox() {
                    //If patches exists, we have to drop it
                    if patches.len() > 0 {
                        let mut command = Command::new("git");
                        command.arg("-C").arg(&source_dir);
                        command.arg("reset").arg("--hard");
                        run_command(command, logger)?;
                    }
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

                if !patches.is_empty() || script.is_some() {
                    // Hard reset
                    let mut command = Command::new("git");
                    command.arg("-C").arg(&source_dir);
                    command.arg("reset").arg("--hard");
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
            head_rev
        }
        Some(SourceRecipe::Tar {
            tar,
            blake3,
            patches,
            script,
        }) => {
            let source_tar = recipe_dir.join("source.tar");
            let mut tar_updated = false;
            loop {
                if !source_tar.is_file() {
                    tar_updated = true;
                    download_wget(&tar, &source_tar, logger)?;
                }
                if !check_source {
                    break;
                }
                let source_tar_blake3 = get_blake3(&source_tar, tar_updated && logger.is_none())?;
                if let Some(blake3) = blake3 {
                    if source_tar_blake3 == *blake3 {
                        break;
                    }
                    if tar_updated {
                        return Err(format!(
                            "The downloaded tar blake3 '{source_tar_blake3}' is not equal to blake3 in recipe.toml"
                        ));
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
            }
            blake3.clone().unwrap_or("no_tar_blake3_hash_info".into())
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
            "local_source".into()
        }
    };

    if let BuildKind::Cargo {
        package_path,
        cargoflags: _,
    } = &recipe.recipe.build.kind
    {
        // TODO: No need to fetch if !check_source and already fetched?
        fetch_cargo(&source_dir, package_path.as_ref(), logger)?;
    }

    fetch_apply_source_info(recipe, ident)?;

    Ok(source_dir)
}

pub(crate) fn fetch_make_symlink(source_dir: &PathBuf, same_as: &String) -> Result<(), String> {
    let target_dir = Path::new(same_as).join("source");
    if !source_dir.is_symlink() {
        if source_dir.is_dir() {
            return Err(format!(
                "'{dir}' is a directory, but recipe indicated a symlink. \n\
                        try removing '{dir}' if you haven't made any changes that would be lost",
                dir = source_dir.display(),
            ));
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
) -> Result<CookRecipe, String> {
    let canon_dir = Path::new(recipe_dir).join(same_as);
    if canon_dir
        .to_str()
        .unwrap()
        .chars()
        .filter(|c| *c == '/')
        .count()
        > 50
    {
        return Err(format!("Infinite loop detected"));
    }
    if !canon_dir.exists() {
        return Err(format!("'{dir}' is not exists.", dir = canon_dir.display()));
    }
    CookRecipe::from_path(canon_dir.as_path(), true, is_host)
        .map_err(|e| format!("Unable to load {dir}: {e:?}", dir = canon_dir.display()))
}

pub(crate) fn fetch_extract_tar(
    source_tar: PathBuf,
    source_dir_tmp: &PathBuf,
    logger: &PtyOut,
) -> Result<(), String> {
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
    package_path: Option<&String>,
    logger: &PtyOut,
) -> Result<(), String> {
    let mut source_dir = source_dir.clone();
    if let Some(package_path) = package_path {
        source_dir = source_dir.join(package_path);
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

fn get_remote_url(name: &str, ext: &str) -> String {
    return format!(
        "{}/{}/{}.{}",
        REMOTE_PKG_SOURCE,
        redoxer::target(),
        name,
        ext
    );
}

fn get_pubkey_url() -> String {
    return format!("{}/id_ed25519.pub.toml", REMOTE_PKG_SOURCE);
}

pub fn fetch_remote(
    recipe_dir: &Path,
    recipe: &CookRecipe,
    offline_mode: bool,
    logger: &PtyOut,
) -> Result<(), String> {
    let target_dir = create_target_dir(recipe_dir, recipe.target)?;
    let source_pubkey = target_dir.join("id_ed25519.pub.toml");
    if !offline_mode {
        download_wget(&get_pubkey_url(), &source_pubkey, logger)?;
    } else {
        offline_check_exists(&source_pubkey)?;
    }

    let packages = recipe.recipe.get_packages_list();

    let name = recipe_dir
        .file_name()
        .ok_or("Unable to get recipe name")?
        .to_str()
        .unwrap();

    for package in packages {
        let (_, source_pkgar, source_toml) = package_source_paths(package, &target_dir);
        let source_name = get_package_name(name, package);

        if !offline_mode {
            //TODO: Check freshness
            download_wget(
                &get_remote_url(&source_name, "pkgar"),
                &source_pkgar,
                logger,
            )?;
            download_wget(&get_remote_url(&source_name, "toml"), &source_toml, logger)?;
        } else {
            offline_check_exists(&source_pkgar)?;
            offline_check_exists(&source_toml)?;
        }

        // guaranteed to exist once
        if package.is_none() {
            let mut file = File::open(&source_toml)
                .map_err(|e| format!("Unable to open source.toml: {e:?}"))?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)
                .map_err(|e| format!("Unable to read source.toml: {e:?}"))?;

            let pkg_toml = pkg::Package::from_toml(&contents)
                .map_err(|e| format!("Unable to parse source.toml: {e:?}"))?;

            fetch_apply_source_info_from_remote(
                recipe,
                &SourceIdentifier {
                    commit_identifier: pkg_toml.commit_identifier.clone(),
                    source_identifier: pkg_toml.source_identifier.clone(),
                    time_identifier: pkg_toml.time_identifier.clone(),
                    ..Default::default()
                },
            )?;
        }
    }

    Ok(())
}

pub(crate) fn fetch_is_patches_newer(
    recipe_dir: &Path,
    patches: &Vec<String>,
    source_dir: &PathBuf,
) -> Result<bool, String> {
    // don't check source files inside as it can be mixed with user patches
    let source_time = modified(&source_dir)?;
    for patch_name in patches {
        let patch_file = recipe_dir.join(patch_name);
        if !patch_file.is_file() {
            return Err(format!(
                "failed to find patch file '{}'",
                patch_file.display()
            ));
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
) -> Result<(), String> {
    for patch_name in patches {
        let patch_file = recipe_dir.join(patch_name);
        if !patch_file.is_file() {
            return Err(format!(
                "failed to find patch file '{}'",
                patch_file.display()
            ));
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
) -> Result<(), String> {
    let ident = crate::cook::ident::get_ident();
    let info = pkg::package::SourceIdentifier {
        commit_identifier: ident.commit.to_string(),
        time_identifier: ident.time.to_string(),
        source_identifier: source_identifier,
    };

    fetch_apply_source_info_from_remote(&recipe, &info)
}

pub(crate) fn fetch_apply_source_info_from_remote(
    recipe: &CookRecipe,
    info: &pkg::package::SourceIdentifier,
) -> Result<(), String> {
    let target_dir = create_target_dir(&recipe.dir, recipe.target)?;
    let source_toml_path = target_dir.join("source_info.toml");
    serialize_and_write(&source_toml_path, &info)?;
    Ok(())
}

pub fn fetch_get_source_info(recipe: &CookRecipe) -> Result<SourceIdentifier, String> {
    let target_dir = recipe.target_dir();
    let source_toml_path = target_dir.join("source_info.toml");
    let toml_content = fs::read_to_string(source_toml_path)
        .map_err(|e| format!("Unable to read source_info.toml: {:?}", e))?;
    let parsed = toml::from_str(&toml_content)
        .map_err(|e| format!("Unable to parse source_info.toml: {:?}", e))?;
    Ok(parsed)
}
