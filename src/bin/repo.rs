use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::{env, fs};

use anyhow::{Context, anyhow};

// A repo manager, to replace repo.sh

const REPO_HELP_STR: &str = r#"
    Usage: repo <command> [flags] <recipe1> <recipe2> ...

    command list:
        fetch     download recipe sources
        cook      build recipe packages
        unfetch   delete recipe sources
        clean     delete recipe artifacts
        push      extract package into sysroot
    
    common flags:
        --cookbook=<cookbook_dir>  the "recipes" folder, default to $PWD/recipes
        --repo=<repo_dir>          the "repo" folder, default to $PWD/repo
        --sysroot=<sysroot_dir>    the "root" folder used for "push" command
            For Redox, defaults to "/", else default to $PWD/sysroot
    
    cook flags:
        --with-package-deps        include package deps
        --offline                  prefer to not use network
        --nonstop                  keep running even a recipe build failed
        --all                      apply to all recipes in <cookbook_dir>
        -q, --quiet                surpress build logs unless error
"#;

struct CliConfig {
    cookbook_dir: PathBuf,
    repo_dir: PathBuf,
    sysroot_dir: PathBuf,
    with_package_deps: bool,
    offline: bool,
    nonstop: bool,
    all: bool,
    quiet: bool,
}

impl CliConfig {
    fn new() -> Result<Self, std::io::Error> {
        let current_dir = env::current_dir()?;
        Ok(CliConfig {
            cookbook_dir: current_dir.join("recipes"),
            repo_dir: current_dir.join("repo"),
            sysroot_dir: if cfg!(target_os = "redox") {
                PathBuf::from("/")
            } else {
                current_dir.join("sysroot")
            },
            with_package_deps: false,
            offline: false,
            nonstop: false,
            all: false,
            quiet: false,
        })
    }
}

fn main() {
    main_inner().unwrap();
}

fn main_inner() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        println!("{}", REPO_HELP_STR);
        process::exit(1);
    }

    let mut config = CliConfig::new()?;
    let mut command: Option<String> = None;
    let mut recipe_paths: BTreeSet<PathBuf> = BTreeSet::new();

    for arg in args {
        if arg.starts_with("--") {
            if let Some((key, value)) = arg.split_once('=') {
                match key {
                    "--cookbook" => config.cookbook_dir = PathBuf::from(value),
                    "--repo" => config.repo_dir = PathBuf::from(value),
                    "--sysroot" => config.sysroot_dir = PathBuf::from(value),
                    _ => {
                        eprintln!("Error: Unknown flag with value: {}", arg);
                        process::exit(1);
                    }
                }
            } else {
                match arg.as_str() {
                    "--with-package-deps" => config.with_package_deps = true,
                    "--offline" => config.offline = true,
                    "--nonstop" => config.nonstop = true,
                    "--all" => config.all = true,
                    "--quiet" => config.quiet = true,
                    _ => {
                        eprintln!("Error: Unknown flag: {}", arg);
                        process::exit(1);
                    }
                }
            }
        } else if arg.starts_with('-') {
            match arg.as_str() {
                "-q" => config.quiet = true,
                _ => {
                    eprintln!("Error: Unknown flag: {}", arg);
                    process::exit(1);
                }
            }
        } else if command.is_none() {
            // The first non-flag argument is the command
            command = Some(arg);
        } else {
            // Subsequent non-flag arguments are recipe names
            if let Some(path) = pkg::recipes::find(&arg) {
                recipe_paths.insert(path.to_owned());
            } else {
                panic!("Error: recipe not found '{arg}'");
            }
        }
    }

    let command = command.ok_or("Error: No command specified.").unwrap();

    if !config.all && recipe_paths.is_empty() {
        panic!("Error: No recipe names provided and --all flag was not used.");
    }
    if config.all && !recipe_paths.is_empty() {
        panic!("Error: Cannot specify recipe names when using the --all flag.");
    }

    if config.all {
        recipe_paths = pkg::recipes::list("");
    }

    for recipe_path in &recipe_paths {
        match command.as_str() {
            "fetch" => handle_fetch(recipe_path, &config)?,
            "cook" => handle_cook(recipe_path, &config)?,
            "unfetch" => handle_unfetch(recipe_path, &config)?,
            "clean" => handle_clean(recipe_path, &config)?,
            "push" => handle_push(recipe_path, &config)?,
            _ => {
                eprintln!("Error: Unknown command '{}'\n", command);
                println!("{}", REPO_HELP_STR);
                process::exit(1);
            }
        }
    }

    println!(
        "\nCommand '{}' completed for all specified recipes.",
        command
    );
    Ok(())
}

fn handle_fetch(recipe_path: &Path, config: &CliConfig) -> anyhow::Result<()> {
    let mut cmd = Command::new("cook");
    cmd.arg("--fetch-only");
    if config.with_package_deps {
        cmd.arg("--with-package-deps");
    }
    if config.offline {
        cmd.arg("--offline");
    }
    if config.quiet {
        cmd.arg("--quiet");
    }
    cmd.arg(recipe_path);
    let status = cmd.status().context("Failed to execute cook command")?;
    if !status.success() && !config.nonstop {
        return Err(anyhow!(
            "Cook command failed for recipe '{}' with exit code: {}",
            recipe_path.display(),
            status.code().unwrap_or(1)
        ));
    }
    Ok(())
}

fn handle_cook(recipe_path: &Path, config: &CliConfig) -> anyhow::Result<()> {
    let mut cmd = Command::new("cook");
    cmd.arg(recipe_path);
    if config.with_package_deps {
        cmd.arg("--with-package-deps");
    }
    if config.offline {
        cmd.arg("--offline");
    }
    if config.quiet {
        cmd.arg("--quiet");
    }
    let status = cmd.status().context("Failed to execute cook command")?;
    if !status.success() && !config.nonstop {
        return Err(anyhow!(
            "Cook command failed for recipe '{}' with exit code: {}",
            recipe_path.display(),
            status.code().unwrap_or(1)
        ));
    }
    Ok(())
}

fn handle_unfetch(recipe_path: &Path, _config: &CliConfig) -> anyhow::Result<()> {
    let dir = recipe_path.join("source");
    if dir.exists() {
        fs::remove_dir_all(dir).context(format!("failed to delete {}", recipe_path.display()))?;
    }
    Ok(())
}

fn handle_clean(recipe_path: &Path, _config: &CliConfig) -> anyhow::Result<()> {
    let dir = recipe_path.join("target");
    if dir.exists() {
        fs::remove_dir_all(dir).context(format!("failed to delete {}", recipe_path.display()))?;
    }
    Ok(())
}

fn handle_push(recipe_path: &Path, config: &CliConfig) -> anyhow::Result<()> {
    let public_path = "build/id_ed25519.pub.toml";
    pkgar::extract(
        public_path,
        config.sysroot_dir.as_path(),
        config.sysroot_dir.to_str().unwrap(),
    )
    .context(format!(
        "failed to install '{}' in '{}'",
        recipe_path.display(),
        config.sysroot_dir.display(),
    ))
}
