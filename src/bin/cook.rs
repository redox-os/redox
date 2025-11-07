use std::path::Path;
use std::{env, process};

use cookbook::cook::fetch::{fetch, fetch_offline};
use cookbook::cook::fs::create_target_dir;
use cookbook::cook::package::package;
use cookbook::recipe::{CookRecipe, Recipe};
use pkg::PackageName;

use cookbook::config::init_config;
use cookbook::cook::cook_build::build;
use termion::{color, style};

fn cook(
    recipe_dir: &Path,
    name: &PackageName,
    recipe: &Recipe,
    is_deps: bool,
    fetch_only: bool,
    is_offline: bool,
) -> Result<(), String> {
    let source_dir = match is_offline {
        true => fetch_offline(recipe_dir, recipe, &None),
        false => fetch(recipe_dir, recipe, &None),
    }
    .map_err(|err| format!("failed to fetch: {}", err))?;

    if fetch_only {
        return Ok(());
    }

    let target_dir = create_target_dir(recipe_dir)?;

    let (stage_dir, auto_deps) = build(
        recipe_dir,
        &source_dir,
        &target_dir,
        name,
        recipe,
        is_offline,
        !is_deps,
        &None,
    )
    .map_err(|err| format!("failed to build: {}", err))?;

    package(&stage_dir, &target_dir, name, recipe, &auto_deps, &None)
        .map_err(|err| format!("failed to package: {}", err))?;

    Ok(())
}

fn main() {
    init_config();
    let mut matching = true;
    let mut dry_run = false;
    let mut fetch_only = false;
    let mut with_package_deps = false;
    let mut quiet = false;
    let mut nonstop = false;
    let mut is_offline = false;
    let mut recipe_names = Vec::new();
    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--" if matching => matching = false,
            "-d" | "--dry-run" if matching => dry_run = true,
            "--with-package-deps" if matching => with_package_deps = true,
            "--fetch-only" if matching => fetch_only = true,
            "-q" | "--quiet" if matching => quiet = true,
            "--nonstop" => nonstop = true,
            "--offline" => is_offline = true,
            _ => recipe_names.push(arg.try_into().expect("Invalid package name")),
        }
    }

    if with_package_deps {
        recipe_names = match CookRecipe::get_package_deps_recursive(&recipe_names, true) {
            Ok(ok) => ok,
            Err(err) => {
                eprintln!(
                    "{}{}cook - error:{}{} {}",
                    style::Bold,
                    color::Fg(color::AnsiValue(196)),
                    color::Fg(color::Reset),
                    style::Reset,
                    err,
                );
                process::exit(1);
            }
        };
    }

    let recipes =
        match CookRecipe::get_build_deps_recursive(&recipe_names, true, !with_package_deps) {
            Ok(ok) => ok,
            Err(err) => {
                eprintln!(
                    "{}{}cook - error:{}{} {}",
                    style::Bold,
                    color::Fg(color::AnsiValue(196)),
                    color::Fg(color::Reset),
                    style::Reset,
                    err,
                );
                process::exit(1);
            }
        };

    for recipe in recipes {
        if !quiet {
            eprintln!(
                "{}{}cook - {}{}{}",
                style::Bold,
                color::Fg(color::AnsiValue(215)),
                recipe.name,
                color::Fg(color::Reset),
                style::Reset,
            );
        }

        let res = if dry_run {
            if !quiet {
                eprintln!("DRY RUN: {:#?}", recipe.recipe);
            }
            Ok(())
        } else {
            cook(
                &recipe.dir,
                &recipe.name,
                &recipe.recipe,
                recipe.is_deps,
                fetch_only,
                is_offline,
            )
        };

        match res {
            Ok(()) => {
                if !quiet {
                    eprintln!(
                        "{}{}cook - {} - successful{}{}",
                        style::Bold,
                        color::Fg(color::AnsiValue(46)),
                        recipe.name,
                        color::Fg(color::Reset),
                        style::Reset,
                    );
                }
            }
            Err(err) => {
                eprintln!(
                    "{}{}cook - {} - error:{}{} {}",
                    style::Bold,
                    color::Fg(color::AnsiValue(196)),
                    recipe.name,
                    color::Fg(color::Reset),
                    style::Reset,
                    err,
                );
                if !nonstop {
                    process::exit(1);
                }
            }
        }
    }
}
