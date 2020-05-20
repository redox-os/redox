use cookbook::blake3::blake3_progress;
use cookbook::recipe::{Recipe, SourceRecipe, BuildRecipe};
use cookbook::sha256::sha256_progress;
use std::{
    env,
    fs,
    path::Path,
    process::{self, Command},
};
use termion::{color, style};

fn create_dir_clean(dir: &Path) -> Result<(), String> {
    if dir.is_dir() {
        // Remove previous directory
        fs::remove_dir_all(&dir).map_err(|err| format!(
            "failed to remove '{}': {}\n{:?}",
            dir.display(),
            err,
            err
        ))?;
    }
    // directory
    fs::create_dir(&dir).map_err(|err| format!(
        "failed to create '{}': {}\n{:?}",
        dir.display(),
        err,
        err
    ))
}

fn rename(src: &Path, dst: &Path) -> Result<(), String> {
    fs::rename(src, dst).map_err(|err| format!(
        "failed to rename '{}' to '{}': {}\n{:?}",
        src.display(),
        dst.display(),
        err,
        err
    ))
}

fn run_command(mut command: process::Command) -> Result<(), String> {
    let status = command.status().map_err(|err| format!(
        "failed to run {:?}: {}\n{:#?}",
        command,
        err,
        err
    ))?;

    if ! status.success() {
        return Err(format!(
            "failed to run {:?}: exited with status {}",
            command,
            status
        ));
    }

    Ok(())
}

fn fetch(source: &SourceRecipe, recipe_dir: &Path) -> Result<(), String> {
    let source_dir = recipe_dir.join("source");
    match source {
        SourceRecipe::Git { git, upstream, branch, rev } => {
            if ! source_dir.is_dir() {
                // Create source.tmp
                let source_dir_tmp = recipe_dir.join("source.tmp");
                create_dir_clean(&source_dir_tmp)?;

                // Clone the repository to source.tmp
                let mut command = Command::new("git");
                command.arg("clone").arg("--recursive").arg(&git);
                if let Some(branch) = branch {
                    command.arg("--branch").arg(&branch);
                }
                command.arg(&source_dir);
                run_command(command)?;

                // Move source.tmp to source atomically
                rename(&source_dir_tmp, &source_dir)?;
            } else {
                // Reset origin
                let mut command = Command::new("git");
                command.arg("-C").arg(&source_dir);
                command.arg("remote").arg("set-url").arg("origin").arg(&git);
                run_command(command)?;

                // Fetch origin
                let mut command = Command::new("git");
                command.arg("-C").arg(&source_dir);
                command.arg("fetch").arg("origin");
                run_command(command)?;
            }

            if let Some(upstream) = upstream {
                //TODO: set upstream URL
                // git remote set-url upstream "$GIT_UPSTREAM" &> /dev/null ||
                // git remote add upstream "$GIT_UPSTREAM"
                // git fetch upstream
            }

            if let Some(rev) = rev {
                // Check out specified revision
                let mut command = Command::new("git");
                command.arg("-C").arg(&source_dir);
                command.arg("checkout").arg(&rev);
                run_command(command)?;
            } else {
                //TODO: complicated stuff to check and reset branch to origin
                // ORIGIN_BRANCH="$(git branch --remotes | grep '^  origin/HEAD -> ' | cut -d ' ' -f 5-)"
                // if [ -n "$BRANCH" ]
                // then
                //     ORIGIN_BRANCH="origin/$BRANCH"
                // fi
                //
                // if [ "$(git rev-parse HEAD)" != "$(git rev-parse $ORIGIN_BRANCH)" ]
                // then
                //     git checkout -B "$(echo "$ORIGIN_BRANCH" | cut -d / -f 2-)" "$ORIGIN_BRANCH"
                // fi
            }

            // Sync submodules URL
            let mut command = Command::new("git");
            command.arg("-C").arg(&source_dir);
            command.arg("submodule").arg("sync").arg("--recursive");
            run_command(command)?;

            // Update submodules
            let mut command = Command::new("git");
            command.arg("-C").arg(&source_dir);
            command.arg("submodule").arg("update").arg("--init").arg("--recursive");
            run_command(command)?;
        },
        SourceRecipe::Tar { tar, blake3, sha256 } => {
            if ! source_dir.is_dir() {
                // Download tar
                //TODO: replace wget
                let source_tar = recipe_dir.join("source.tar");
                if ! source_tar.is_file() {
                    let source_tar_tmp = recipe_dir.join("source.tar.tmp");

                    let mut command = Command::new("wget");
                    command.arg(&tar);
                    command.arg("-O").arg(&source_tar_tmp);
                    run_command(command)?;

                    // Move source.tar.tmp to source.tar atomically
                    rename(&source_tar_tmp, &source_tar)?;
                }

                if let Some(blake3) = blake3 {
                    //TODO
                    // Calculate blake3
                    let source_tar_blake3 = blake3_progress(&source_tar).map_err(|err| format!(
                        "failed to calculate blake3 of '{}': {}\n{:?}",
                        source_tar.display(),
                        err,
                        err
                    ))?;

                    // Check if it matches recipe
                    if &source_tar_blake3 != blake3 {
                        return Err(format!(
                            "calculated blake3 '{}' does not match recipe blake3 '{}'",
                            source_tar_blake3,
                            blake3
                        ));
                    }
                }

                if let Some(sha256) = sha256 {
                    // Calculate sha256
                    let source_tar_sha256 = sha256_progress(&source_tar).map_err(|err| format!(
                        "failed to calculate sha256 of '{}': {}\n{:?}",
                        source_tar.display(),
                        err,
                        err
                    ))?;

                    // Check if it matches recipe
                    if &source_tar_sha256 != sha256 {
                        return Err(format!(
                            "calculated sha256 '{}' does not match recipe sha256 '{}'",
                            source_tar_sha256,
                            sha256
                        ));
                    }
                }

                // Create source.tmp
                let source_dir_tmp = recipe_dir.join("source.tmp");
                create_dir_clean(&source_dir_tmp)?;

                // Extract tar to source.tmp
                //TODO: use tar crate (how to deal with compression?)
                let mut command = Command::new("tar");
                command.arg("--extract");
                command.arg("--verbose");
                command.arg("--file").arg(&source_tar);
                command.arg("--directory").arg(&source_dir_tmp);
                run_command(command)?;

                // Move source.tmp to source atomically
                rename(&source_dir_tmp, &source_dir)?;
            }
        }
    }

    Ok(())
}

fn cook(recipe_name: &str) -> Result<(), String> {
    //TODO: sanitize recipe name?
    let recipe_dir = Path::new("recipes").join(recipe_name);
    if ! recipe_dir.is_dir() {
        return Err(format!(
            "failed to find recipe directory '{}'",
            recipe_dir.display()
        ));
    }

    let recipe_file = recipe_dir.join("recipe.toml");
    if ! recipe_file.is_file() {
        return Err(format!(
            "failed to find recipe file '{}'",
            recipe_file.display()
        ));
    }

    let recipe_toml = fs::read_to_string(&recipe_file).map_err(|err| format!(
        "failed to read recipe file '{}': {}\n{:#?}",
        recipe_file.display(),
        err,
        err
    ))?;

    let recipe: Recipe = toml::from_str(&recipe_toml).map_err(|err| format!(
        "failed to parse recipe file '{}': {}\n{:#?}",
        recipe_file.display(),
        err,
        err
    ))?;

    fetch(&recipe.source, &recipe_dir).map_err(|err| format!(
        "failed to fetch: {}",
        err
    ))?;

    Ok(())
}

fn main() {
    let mut matching = true;
    let mut quiet = false;
    let mut recipe_names = Vec::new();
    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--" if matching => matching = false,
            "-q" | "--quiet" if matching => quiet = true,
            _ => recipe_names.push(arg),
        }
    }

    for recipe_name in recipe_names.iter() {
        if ! quiet {
            eprintln!(
                "{}{}cook - {}{}{}",
                style::Bold,
                color::Fg(color::AnsiValue(215)),
                recipe_name,
                color::Fg(color::Reset),
                style::Reset,
            );
        }

        match cook(recipe_name) {
            Ok(()) => {
                if ! quiet {
                    eprintln!(
                        "{}{}cook - {} - successful{}{}",
                        style::Bold,
                        color::Fg(color::AnsiValue(46)),
                        recipe_name,
                        color::Fg(color::Reset),
                        style::Reset,
                    );
                }
            },
            Err(err) => {
                eprintln!(
                    "{}{}cook - {} - error:{}{} {}",
                    style::Bold,
                    color::Fg(color::AnsiValue(196)),
                    recipe_name,
                    color::Fg(color::Reset),
                    style::Reset,
                    err,
                );
                process::exit(1);
            }
        }
    }
}
