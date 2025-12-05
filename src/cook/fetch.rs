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
use crate::recipe::Recipe;
use crate::{blake3, recipe::SourceRecipe};
use std::fs;
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

pub fn fetch_offline(
    recipe_dir: &Path,
    recipe: &Recipe,
    logger: &PtyOut,
) -> Result<PathBuf, String> {
    let source_dir = recipe_dir.join("source");
    if recipe.build.kind == BuildKind::None {
        // the build function doesn't need source dir exists
        return Ok(source_dir);
    }
    if recipe.build.kind == BuildKind::Remote {
        fetch_remote(recipe_dir, recipe, true, logger)?;
        return Ok(source_dir);
    }

    match &recipe.source {
        Some(SourceRecipe::Path { path: _ }) | None => {
            return fetch(recipe_dir, recipe, &None, logger);
        }
        Some(SourceRecipe::SameAs { same_as: _ }) => {
            return fetch(recipe_dir, recipe, &None, logger);
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
        }
    }

    Ok(source_dir)
}

pub fn fetch(
    recipe_dir: &Path,
    recipe: &Recipe,
    update_tag: &Option<PathBuf>,
    logger: &PtyOut,
) -> Result<PathBuf, String> {
    let source_dir = recipe_dir.join("source");
    if recipe.build.kind == BuildKind::None {
        // the build function doesn't need source dir exists
        return Ok(source_dir);
    }
    if recipe.build.kind == BuildKind::Remote {
        fetch_remote(recipe_dir, recipe, false, logger)?;
        return Ok(source_dir);
    }

    match &recipe.source {
        Some(SourceRecipe::SameAs { same_as }) => {
            let (canon_dir, recipe) = fetch_resolve_canon(recipe_dir, &same_as)?;
            // recursively fetch
            fetch(&canon_dir, &recipe, update_tag, logger)?;
            fetch_make_symlink(&source_dir, &same_as)?;
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
            let origin_commit = if !source_dir.is_dir() {
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

                None
            } else {
                let source_git_dir = source_dir.join(".git");
                if !source_git_dir.is_dir() {
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

                if update_tag.is_some() {
                    Some(get_git_head_rev(&source_dir)?)
                } else {
                    // not needed
                    None
                }
            };

            if let Some(_upstream) = upstream {
                //TODO: set upstream URL
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

            if let Some(update_tag) = update_tag {
                if match origin_commit {
                    Some(origin_commit) => origin_commit != get_git_head_rev(&source_dir)?,
                    None => true,
                } {
                    let mut command = Command::new("touch");
                    command.arg(&update_tag);
                    run_command(command, logger)?;
                };
            }

            fetch_apply_patches(recipe_dir, patches, script, &source_dir, logger)?;
        }
        Some(SourceRecipe::Tar {
            tar,
            blake3,
            patches,
            script,
        }) => {
            let source_tar = recipe_dir.join("source.tar");
            let mut tar_updated = false;
            while {
                if !source_tar.is_file() {
                    tar_updated = true;
                    download_wget(&tar, &source_tar, logger)?;
                }
                let source_tar_blake3 = get_blake3(&source_tar, tar_updated && logger.is_none())?;
                if let Some(blake3) = blake3 {
                    if source_tar_blake3 != *blake3 {
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
                        true
                    } else {
                        false
                    }
                } else {
                    //TODO: set blake3 hash on the recipe with something like "cook fix"
                    log_to_pty!(
                        logger,
                        "WARNING: set blake3 for '{}' to '{}'",
                        source_tar.display(),
                        source_tar_blake3
                    );
                    false
                }
            } {}
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
        }
    }

    if let BuildKind::Cargo {
        package_path,
        cargoflags: _,
    } = &recipe.build.kind
    {
        fetch_cargo(&source_dir, package_path.as_ref(), logger)?;
    }

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
) -> Result<(PathBuf, Recipe), String> {
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
    let recipe_path = canon_dir.join("recipe.toml");
    let recipe_str = fs::read_to_string(&recipe_path)
        .map_err(|e| format!("unable to read {path}: {e}", path = recipe_path.display()))?;
    let recipe: Recipe = toml::from_str(&recipe_str)
        .map_err(|e| format!("Unable to parse {path}: {e}", path = recipe_path.display()))?;
    Ok((canon_dir, recipe))
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

    let mut command = if is_redox() {
        Command::new("cargo")
    } else {
        let cookbook_redoxer = Path::new("target/release/cookbook_redoxer")
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
    recipe: &Recipe,
    offline_mode: bool,
    logger: &PtyOut,
) -> Result<(), String> {
    // TODO: allow download to host target (waiting for build server to have them)
    let target = redoxer::target();
    let target_dir = create_target_dir(recipe_dir, target)?;
    let source_pubkey = target_dir.join("id_ed25519.pub.toml");
    if !offline_mode {
        download_wget(&get_pubkey_url(), &source_pubkey, logger)?;
    } else {
        offline_check_exists(&source_pubkey)?;
    }

    let packages = recipe.get_packages_list();

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
