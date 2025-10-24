use std::path::PathBuf;
use std::process;
use std::str::FromStr;
use std::{env, fs};

use anyhow::{Context, anyhow, bail};
use cookbook::WALK_DEPTH;
use cookbook::config::{CookConfig, get_config, init_config};
use cookbook::cook::cook_build::build;
use cookbook::cook::fetch::{fetch, fetch_offline};
use cookbook::cook::fs::create_target_dir;
use cookbook::cook::package::package;
use cookbook::recipe::CookRecipe;
use pkg::PackageName;
use pkg::package::PackageError;

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
    all: bool,
    cook: CookConfig,
}

#[derive(PartialEq)]
enum CliCommand {
    Fetch,
    Cook,
    Unfetch,
    Clean,
    Push,
}

impl FromStr for CliCommand {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fetch" => Ok(CliCommand::Fetch),
            "cook" => Ok(CliCommand::Cook),
            "unfetch" => Ok(CliCommand::Unfetch),
            "clean" => Ok(CliCommand::Clean),
            "push" => Ok(CliCommand::Push),
            _ => Err(anyhow!("Unknown command '{}'", s)),
        }
    }
}

impl ToString for CliCommand {
    fn to_string(&self) -> String {
        match self {
            CliCommand::Fetch => "fetch".to_string(),
            CliCommand::Cook => "cook".to_string(),
            CliCommand::Unfetch => "unfetch".to_string(),
            CliCommand::Clean => "clean".to_string(),
            CliCommand::Push => "push".to_string(),
        }
    }
}

impl CliConfig {
    fn new() -> Result<Self, std::io::Error> {
        let current_dir = env::current_dir()?;
        Ok(CliConfig {
            //FIXME: This config is unused as redox-pkg harcoded this to $PWD/recipes
            cookbook_dir: current_dir.join("recipes"),
            repo_dir: current_dir.join("repo"),
            sysroot_dir: if cfg!(target_os = "redox") {
                PathBuf::from("/")
            } else {
                current_dir.join("sysroot")
            },
            with_package_deps: false,
            cook: get_config().cook.clone(),
            all: false,
        })
    }
}

fn main() {
    init_config();
    main_inner().unwrap();
}

fn main_inner() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        println!("{}", REPO_HELP_STR);
        process::exit(1);
    }

    let (config, command, recipe_names) = parse_args(args)?;

    for recipe in &recipe_names {
        match command {
            CliCommand::Fetch => handle_cook(recipe, &config, true, recipe.is_deps)?,
            CliCommand::Cook => handle_cook(recipe, &config, false, recipe.is_deps)?,
            CliCommand::Unfetch => handle_clean(recipe, &config, true, true)?,
            CliCommand::Clean => handle_clean(recipe, &config, false, true)?,
            CliCommand::Push => handle_push(recipe, &config)?,
        }
    }

    println!(
        "\nCommand '{}' completed for all specified recipes.",
        command.to_string(),
    );
    Ok(())
}

fn parse_args(args: Vec<String>) -> anyhow::Result<(CliConfig, CliCommand, Vec<CookRecipe>)> {
    let mut config = CliConfig::new()?;
    let mut command: Option<String> = None;
    let mut recipe_names: Vec<PackageName> = Vec::new();
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
                    "--all" => config.all = true,
                    _ => {
                        eprintln!("Error: Unknown flag: {}", arg);
                        process::exit(1);
                    }
                }
            }
        } else if arg.starts_with('-') {
            match arg.as_str() {
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
            recipe_names.push(arg.try_into().context("Invalid package name")?);
        }
    }

    let command = command.ok_or(anyhow!("Error: No command specified."))?;
    let command: CliCommand = str::parse(&command)?;
    let recipes = if config.all {
        if !recipe_names.is_empty() {
            bail!("Cannot specify recipe names when using the --all flag.");
        }
        if command == CliCommand::Cook
            || command == CliCommand::Fetch
            || command == CliCommand::Push
        {
            // because read_recipe is false below
            // some recipes on wip folders are invalid anyway
            bail!(
                "Refusing to run an unrealistic command to {} all recipes",
                command.to_string()
            );
        }

        pkg::recipes::list("")
            .iter()
            .map(|f| CookRecipe::from_path(f, false))
            .collect::<Result<Vec<CookRecipe>, PackageError>>()?
    } else {
        if recipe_names.is_empty() {
            bail!("Error: No recipe names provided and --all flag was not used.");
        }
        if config.with_package_deps {
            recipe_names = CookRecipe::get_package_deps_recursive(&recipe_names, WALK_DEPTH)
                .context("failed get package deps")?;
        }

        CookRecipe::get_build_deps_recursive(&recipe_names, !config.with_package_deps)?
    };

    Ok((config, command, recipes))
}

fn handle_cook(
    recipe: &CookRecipe,
    config: &CliConfig,
    fetch_only: bool,
    is_deps: bool,
) -> anyhow::Result<()> {
    let recipe_dir = &recipe.dir;
    let source_dir = match config.cook.offline {
        true => fetch_offline(recipe_dir, &recipe.recipe),
        false => fetch(recipe_dir, &recipe.recipe),
    }
    .map_err(|e| anyhow!(e))?;

    if fetch_only {
        return Ok(());
    }

    let target_dir = create_target_dir(recipe_dir).map_err(|e| anyhow!(e))?;

    let (stage_dir, auto_deps) = build(
        recipe_dir,
        &source_dir,
        &target_dir,
        &recipe.name,
        &recipe.recipe,
        config.cook.offline,
        !is_deps,
    )
    .map_err(|err| anyhow!("failed to build: {}", err))?;

    package(
        &stage_dir,
        &target_dir,
        &recipe.name,
        &recipe.recipe,
        &auto_deps,
    )
    .map_err(|err| anyhow!("failed to package: {}", err))?;

    Ok(())
}

fn handle_clean(
    recipe: &CookRecipe,
    _config: &CliConfig,
    source: bool,
    target: bool,
) -> anyhow::Result<()> {
    let dir = recipe.dir.join("target");
    if dir.exists() && target {
        fs::remove_dir_all(&dir).context(format!("failed to delete {}", dir.display()))?;
    }
    let dir = recipe.dir.join("source");
    if dir.exists() && source {
        fs::remove_dir_all(&dir).context(format!("failed to delete {}", dir.display()))?;
    }
    Ok(())
}

fn handle_push(recipe: &CookRecipe, config: &CliConfig) -> anyhow::Result<()> {
    let public_path = "build/id_ed25519.pub.toml";
    let archive_path = config.repo_dir.join(recipe.name.as_str());
    pkgar::extract(
        public_path,
        archive_path.as_path(),
        config.sysroot_dir.to_str().unwrap(),
    )
    .context(format!(
        "failed to install '{}' in '{}'",
        archive_path.display(),
        config.sysroot_dir.display(),
    ))
}
