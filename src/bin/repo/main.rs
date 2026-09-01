use cookbook::config::{CookConfig, CookLockOpt, get_config, init_config};
use cookbook::cook::cook_build::{build, get_stage_dirs, remove_stage_dir};
use cookbook::cook::fetch::{FetchResult, fetch, fetch_offline};
use cookbook::cook::fs::{
    create_dir, create_target_dir, get_git_commit_date, get_git_head_rev, get_git_rev_before_date,
    remove_all, run_command,
};
use cookbook::cook::package::{package, package_handle_push};
use cookbook::cook::pty::{PtyOut, flush_pty, write_to_pty};
use cookbook::cook::tree::{self, DisplayOptions, TreeData, TreeItem, TreeOptions, WalkTreeEntry};
use cookbook::cook::tui::join_logs;
use cookbook::cook::{fetch_repo, ident};
use cookbook::recipe::{
    CookRecipe, SourceRecipe, recipes_flatten_package_names, recipes_mark_as_deps,
};
use cookbook::{Error, Result, staged_pkg};
use pkg::{PackageName, PackageState};
use redox_installer::PackageConfig;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::{Write, stderr};
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use std::sync::{OnceLock, mpsc};
use std::{env, fs};
use std::{process, thread};
use termion::{color, style};

mod app;
use app::*;

// A repo manager, to replace repo.sh

const REPO_HELP_STR: &str = r#"
    Usage: repo <command> [flags] <recipe1> <recipe2> ...

    command list:
        fetch        download recipe sources
        cook         build recipe packages
        unfetch      delete recipe sources
        clean        delete recipe artifacts
        clean-target delete recipe artifacts for one target
        push         extract package into sysroot
        repo-list    show list of recipes
        cook-list    show list of recipes to build
        push-list    show list of recipes to package
        capture-rev  write lock to git recipes
        change-rule  override rule to recipes
        change-rule-local  override rule to specific recipes

    common flags:
        --cookbook=<cookbook_dir>  the "recipes" folder, default to $PWD/recipes
        --repo=<repo_dir>          the "repo" folder, default to $PWD/repo
        --with-package-deps        include package deps (always implied in push command)
        --all                      apply to all recipes in <cookbook_dir>
        --all-compiled             apply to all compiled recipes in <cookbook_dir>
        --all-binaries             apply to all compiled recipes in <cookbook_dir> that is configured as "binary"
        --category=<category>      apply to all recipes in <cookbook_dir>/<category>
        --filesystem=<filesystem>  override recipes config using installer file
        --repo-binary              override recipes config to use repo_binary
        --sysroot=<sysroot_dir>    used in "push", the "root" dir, default to $PWD/sysroot
        --no-metadata              used in "push", do not write pkgar_head or etc dir
        --display=<format>         used in "*-list", either "name", "path", "csv", "tree"
        --set-rule=<rule>          used in "change-rule", set wanted config rule
        --rollback                 used in "capture-rev", allow git to rollback
        --unset                    used in "capture-rev" and "change-rule", unset locks

    cook env and their defaults:
        CI=                          set to any value to disable TUI
        COOKBOOK_LOGS=               whether to capture build logs (default is !CI)
        COOKBOOK_OFFLINE=false       prevent internet access if possible
                                        ignored when command "fetch" is used
        COOKBOOK_NONSTOP=false       keep running even a recipe build failed
        COOKBOOK_COMPRESSED=false    build packages in compressed format
        COOKBOOK_GIT_TREELESS=false  clone sources as treeless by default
        COOKBOOK_VERBOSE=true        print success/error on each recipe
        COOKBOOK_VERBOSE_CMD=true    add -x to bash build script
        COOKBOOK_CLEAN_BUILD=false   remove build directory before building
        COOKBOOK_CLEAN_TARGET=false  remove target directory after building
        COOKBOOK_WRITE_FILETREE=false whether to write stage files tree
        COOKBOOK_MAKE_JOBS=          override build jobs count from nproc
        COOKBOOK_WEB=false           whether to generate package web files
"#;

#[derive(Clone)]
struct CliConfig {
    cookbook_dir: PathBuf,
    repo_dir: PathBuf,
    sysroot_dir: PathBuf,
    logs_dir: Option<PathBuf>,
    category: Option<PathBuf>,
    filesystem: Option<redox_installer::Config>,
    set_rule: Option<String>,
    display: DisplayOptions,
    unset: bool,
    no_metadata: bool,
    with_rollback: bool,
    with_package_deps: bool,
    all: Option<AllOption>,
    cook: CookConfig,
}

#[derive(PartialEq)]
enum CliCommand {
    Fetch,
    Cook,
    Unfetch,
    Clean,
    CleanTarget,
    Push,
    CookList,
    PushList,
    RepoList,
    CaptureRev,
    ChangeRule,
    ChangeRuleLocal,
}

#[derive(Clone)]
enum AllOption {
    All,
    AllCompiled,
    AllBinaries,
}

impl CliCommand {
    pub fn is_informational(&self) -> bool {
        *self == CliCommand::PushList
            || *self == CliCommand::CookList
            || *self == CliCommand::RepoList
    }
    pub fn is_tree(&self) -> bool {
        self.is_informational()
    }
    pub fn is_change_rule(&self) -> bool {
        *self == CliCommand::ChangeRuleLocal
            || *self == CliCommand::CaptureRev
            || *self == CliCommand::ChangeRule
    }
    pub fn is_building(&self) -> bool {
        *self == CliCommand::Fetch
            || *self == CliCommand::Cook
            || *self == CliCommand::CookList
            || *self == CliCommand::CaptureRev
            || *self == CliCommand::ChangeRule
    }
    pub fn is_pushing(&self) -> bool {
        *self == CliCommand::Push || *self == CliCommand::PushList
    }
    pub fn is_cleaning(&self) -> bool {
        *self == CliCommand::Clean
            || *self == CliCommand::CleanTarget
            || *self == CliCommand::Unfetch
    }
    pub fn to_tree(&self) -> TreeOptions {
        match self {
            CliCommand::CookList => TreeOptions::Cook,
            CliCommand::PushList => TreeOptions::Push,
            CliCommand::RepoList => TreeOptions::Repo,
            _ => unreachable!(),
        }
    }
}

impl FromStr for CliCommand {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "fetch" => Ok(CliCommand::Fetch),
            "cook" => Ok(CliCommand::Cook),
            // alias for repo_editor
            "rebuild" => Ok(CliCommand::Cook),
            "unfetch" => Ok(CliCommand::Unfetch),
            "clean" => Ok(CliCommand::Clean),
            "clean-target" => Ok(CliCommand::CleanTarget),
            "push" => Ok(CliCommand::Push),
            "repo-list" => Ok(CliCommand::RepoList),
            "push-list" => Ok(CliCommand::PushList),
            "cook-list" => Ok(CliCommand::CookList),
            // alias for scripts
            "find" => Ok(CliCommand::RepoList),
            "capture-rev" => Ok(CliCommand::CaptureRev),
            "change-rule" => Ok(CliCommand::ChangeRule),
            "change-rule-local" => Ok(CliCommand::ChangeRuleLocal),
            _ => bail_options_err!("Unknown command {:?}", s),
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
            CliCommand::CleanTarget => "clean-target".to_string(),
            CliCommand::Push => "push".to_string(),
            CliCommand::PushList => "push-list".to_string(),
            CliCommand::CookList => "cook-list".to_string(),
            CliCommand::RepoList => "repo-list".to_string(),
            CliCommand::CaptureRev => "capture-rev".to_string(),
            CliCommand::ChangeRule => "change-rule".to_string(),
            CliCommand::ChangeRuleLocal => "change-rule-local".to_string(),
        }
    }
}

impl CliConfig {
    fn new() -> Result<Self> {
        let current_dir = env::current_dir().map_err(|e| Error::from_io_error(e, "Getting cwd"))?;
        Ok(CliConfig {
            //FIXME: This config is unused as redox-pkg harcoded this to $PWD/recipes
            cookbook_dir: current_dir.join("recipes"),
            repo_dir: current_dir.join("repo"),
            // build dir here is hardcoded in repo_builder as well
            logs_dir: if get_config().cook.logs {
                Some(current_dir.join("build/logs"))
            } else {
                None
            },
            category: None,
            display: DisplayOptions::Tree,
            sysroot_dir: current_dir.join("sysroot"),
            with_package_deps: false,
            cook: get_config().cook.clone(),
            all: None,
            unset: false,
            no_metadata: false,
            filesystem: None,
            with_rollback: false,
            set_rule: None,
        })
    }
}

fn main() {
    init_config();
    if let Err(e) = main_inner() {
        match e {
            Error::Options(e) => eprintln!("{}\n{}", e, REPO_HELP_STR),
            e => eprintln!("{}", e),
        }
        process::exit(1);
    };
}

fn main_inner() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        bail_options_err!("");
    }

    let (config, command, recipes) = parse_args(args)?;
    if command.is_building() || matches!(command, CliCommand::ChangeRuleLocal) {
        ident::init_ident();
    }
    if command == CliCommand::Cook && config.cook.tui {
        match run_tui_cook(config.clone(), recipes.clone()) {
            Ok(TuiApp {
                dump_logs_on_exit: Some((name, err)),
                ..
            }) => {
                let _ = stderr().write(err.as_bytes());
                let _ = stderr().write(b"\n\n");
                print_failed(&command, &name);
                return Err(Error::from("Execution has failed".to_string()));
            }
            Ok(app) => {
                for (recipe, status) in app.recipes {
                    match status {
                        RecipeStatus::Cached => print_cached(&command, &recipe.name),
                        RecipeStatus::Done => print_success(&command, &recipe.name),
                        RecipeStatus::Failed(err) => {
                            let _ = stderr().write(err.as_bytes());
                            let _ = stderr().write(b"\n\n");
                            print_failed(&command, &recipe.name)
                        }
                        _ => unreachable!(),
                    }
                }
            }
            Err(e) => return Err(e),
        }
        return publish_packages(&recipes, &config.repo_dir);
    }
    if command.is_tree() {
        return handle_tree(&recipes, command.to_tree(), &config);
    }
    if command.is_change_rule() {
        return handle_change_rule(&recipes, &config, &command);
    }
    if command == CliCommand::Push {
        return handle_push(&recipes, &config);
    }

    let verbose = config.cook.verbose;
    for recipe in &recipes {
        match repo_inner(&config, &command, recipe) {
            Ok(cached) => {
                if !command.is_informational() {
                    if cached {
                        print_cached(&command, &recipe.name);
                    } else {
                        print_success(&command, &recipe.name);
                    }
                }
            }
            Err(e) => {
                if config.cook.nonstop {
                    if verbose {
                        eprintln!("{}", e);
                    }
                    if let Err(e) = handle_nonstop_fail(recipe) {
                        eprintln!("{}", e)
                    };
                }
                print_failed(&command, &recipe.name);
                if !config.cook.nonstop {
                    return Err(e);
                }
            }
        }
    }

    if command == CliCommand::Cook {
        return publish_packages(&recipes, &config.repo_dir);
    }

    if verbose && recipes.len() > 1 {
        println!(
            "\nCommand '{}' completed for {} recipes.",
            command.to_string(),
            recipes.len()
        );
    }
    Ok(())
}

fn print_failed(command: &CliCommand, recipe: &PackageName) {
    eprintln!(
        "{}{}{} {} - failed {}{}",
        style::Bold,
        color::Fg(color::AnsiValue(196)),
        command.to_string(),
        recipe.as_str(),
        color::Fg(color::Reset),
        style::Reset,
    );
}

fn print_success(command: &CliCommand, recipe: &PackageName) {
    eprintln!(
        "{}{}{} {} - successful{}{}",
        style::Bold,
        color::Fg(color::AnsiValue(46)),
        command.to_string(),
        recipe.as_str(),
        color::Fg(color::Reset),
        style::Reset,
    );
}

fn print_cached(command: &CliCommand, recipe: &PackageName) {
    eprintln!(
        "{}{}{} {} - cached{}{}",
        style::Bold,
        color::Fg(color::AnsiValue(45)),
        command.to_string(),
        recipe.as_str(),
        color::Fg(color::Reset),
        style::Reset,
    );
}

fn repo_inner(config: &CliConfig, command: &CliCommand, recipe: &CookRecipe) -> Result<bool> {
    Ok(match *command {
        CliCommand::Fetch | CliCommand::Cook => {
            let repo_inner_fn = move |logger: &PtyOut| -> Result<bool> {
                let is_cook = *command == CliCommand::Cook;
                let fetch_result = handle_fetch(recipe, config, is_cook, logger)?;
                let cached = if is_cook {
                    handle_cook(recipe, config, fetch_result.source_dir, logger)?
                } else {
                    fetch_result.cached
                };
                Ok(cached)
            };
            let Some(log_path) = &config.logs_dir else {
                return repo_inner_fn(&None);
            };

            let (status_tx, status_rx) = mpsc::channel::<StatusUpdate>();
            let (mut stdout_writer, mut stderr_writer) = setup_logger(&status_tx, &recipe.name);
            let mut app = TuiApp::new(vec![recipe.clone()]);
            app.dump_logs_anyway = config.cook.verbose;
            let dump_fail_logs = !app.dump_logs_anyway;
            let th = thread::spawn(move || {
                while let Ok(update) = status_rx.recv() {
                    match &update {
                        StatusUpdate::CookThreadFinished => break,
                        StatusUpdate::FailCook(r, _) => {
                            let (logs, line) = app.get_recipe_log(&r.name);
                            if let Some(logs) = logs {
                                println!("{}", join_logs(logs, line));
                            }
                        }
                        _ => app.update_status(update),
                    }
                }
            });
            let mut logger = Some((&mut stdout_writer, &mut stderr_writer));
            let result = repo_inner_fn(&logger);
            if let Err(err_ctx) = &result {
                write_to_pty(&logger, &format!("\n{err_ctx}"));
            }
            // successful cached build is not that useful to log
            if !matches!(result, Ok(true)) {
                flush_pty(&mut logger);
                let log_path =
                    log_path.join(format!("{}/{}.log", recipe.target, recipe.name.name()));
                status_tx
                    .send(StatusUpdate::FlushLog(recipe.name.clone(), log_path))
                    .unwrap_or_default();
                if dump_fail_logs && result.is_err() {
                    status_tx
                        .send(StatusUpdate::FailCook(recipe.clone(), "".into()))
                        .unwrap_or_default();
                }
            }
            status_tx
                .send(StatusUpdate::CookThreadFinished)
                .unwrap_or_default();
            let _ = th.join();
            result?
        }
        CliCommand::Unfetch | CliCommand::Clean | CliCommand::CleanTarget => {
            handle_clean(recipe, config, command)?
        }
        _ => unreachable!(),
    })
}

fn publish_packages(recipe_names: &Vec<CookRecipe>, repo_path: &PathBuf) -> Result<()> {
    let repo_bin = env::current_exe()
        .map_err(|e| Error::from_io_error(e, "Getting exe path"))?
        .parent()
        .unwrap()
        .join("repo_builder");
    let mut command = Command::new(repo_bin);
    command
        .arg(repo_path)
        .args(recipe_names.iter().filter_map(|n| {
            if !n.is_deps {
                Some(n.name.as_str())
            } else {
                None
            }
        }));

    run_command(command, &None)
}

fn parse_args(args: Vec<String>) -> Result<(CliConfig, CliCommand, Vec<CookRecipe>)> {
    let mut config = CliConfig::new()?;
    let mut command: Option<String> = None;
    let mut recipe_names: Vec<PackageName> = Vec::new();
    let mut override_filesystem_repo_binary = false;
    for arg in args {
        if arg.starts_with("--") {
            if let Some((key, value)) = arg.split_once('=') {
                match key {
                    "--cookbook" => config.cookbook_dir = PathBuf::from(value),
                    "--repo" => config.repo_dir = PathBuf::from(value),
                    "--sysroot" => config.sysroot_dir = PathBuf::from(value),
                    "--category" => config.category = Some(PathBuf::from(value)),
                    "--set-rule" => config.set_rule = Some(value.into()),
                    "--display" => config.display = DisplayOptions::from_str(value)?,
                    "--filesystem" => {
                        config.filesystem = Some({
                            let r = redox_installer::Config::from_file(&PathBuf::from(value));
                            r.map_err(|e| Error::Other(format!("{:?}", e)))?
                        })
                    }
                    _ => bail_options_err!("Error: Unknown flag with value: {}", arg),
                }
            } else if arg.starts_with("--category-") {
                // to workaround make command limit we provide this option
                config.category = Some(PathBuf::from(
                    arg[("--category-").len()..].replace('.', "/").to_owned(),
                ));
            } else {
                match arg.as_str() {
                    "--repo-binary" => override_filesystem_repo_binary = true,
                    "--with-package-deps" => config.with_package_deps = true,
                    "--no-metadata" => config.no_metadata = true,
                    "--rollback" => config.with_rollback = true,
                    "--unset" => config.unset = true,
                    "--all" => config.all = Some(AllOption::All),
                    "--all-compiled" => config.all = Some(AllOption::AllCompiled),
                    "--all-binaries" => config.all = Some(AllOption::AllBinaries),
                    _ => bail_options_err!("Error: Unknown flag: {}", arg),
                }
            }
        } else if arg.starts_with('-') {
            match arg.as_str() {
                _ => bail_options_err!("Error: Unknown flag: {}", arg),
            }
        } else if command.is_none() {
            // The first non-flag argument is the command
            command = Some(arg);
        } else {
            // Subsequent non-flag arguments are recipe names
            recipe_names.push(arg.try_into().map_err(Error::from)?);
        }
    }

    if let Some(c) = config.category {
        // need to prefix by cookbook dir
        config.category = Some(PathBuf::from("recipes").join(c));
    }
    if let Some(c) = config.logs_dir.as_mut() {
        create_dir(&c.join(redoxer::target()))?;
        create_dir(&c.join(redoxer::host_target()))?;
    }

    let Some(command) = command else {
        bail_options_err!("Error: No command specified");
    };
    let command =
        if command.starts_with("change-rule ") || command.starts_with("change-rule-local ") {
            // repo_editor hack
            let mut split = command.split(' ');
            let cmd: CliCommand = str::parse(split.next().unwrap())?;
            config.set_rule = split.next().map(|s| s.to_string());
            cmd
        } else {
            if command == "find" {
                config.display = DisplayOptions::Path;
            }
            str::parse(&command)?
        };
    if command.is_informational() {
        // avoid extra data that clobber stdout
        config.cook.verbose = false;
    }

    let mut preloaded_recipes: BTreeMap<PackageName, CookRecipe> = BTreeMap::new();

    // TODO: Unindent
    {
        if config.all.is_some() || config.category.is_some() {
            let all_recipes_path = match (&config.all, config.category.is_some()) {
                (Some(AllOption::All), _) | (None, true) => {
                    // everything in recipes
                    staged_pkg::list()
                }
                _ => {
                    // get the list from repo/TARGET/repo.toml
                    staged_pkg::list_repo(&config.repo_dir)?
                }
            };

            let all_recipes_path = match &config.category {
                None => all_recipes_path,
                Some(prefix) => all_recipes_path
                    .into_iter()
                    .filter(|p| p.starts_with(prefix))
                    .collect(),
            };

            if all_recipes_path.is_empty() {
                bail_options_err!(
                    "No recipes found from the combination.\n\
                    Try pass both --all and --category=name"
                );
            }

            for path in all_recipes_path {
                // TODO: Allow selecting recipes from category as host?
                let recipe = match CookRecipe::from_path(&path, !command.is_cleaning(), false) {
                    Ok(recipe) => recipe,
                    Err(_) if matches!(config.all, Some(AllOption::All)) => continue,
                    Err(e) => return Err(e.into()),
                };
                let recipe_name = recipe.name.clone();
                preloaded_recipes.insert(recipe_name.clone(), recipe);
                recipe_names.push(recipe_name);
            }
        } else if recipe_names.is_empty() {
            if let Some(conf) = config.filesystem.as_ref() {
                recipe_names = conf
                    .packages
                    .keys()
                    .filter_map(|k| PackageName::new(k.to_string()).ok())
                    .collect();
            } else {
                bail_options_err!(
                    "Error: No recipe names or filesystem config provided and --all flag was not used."
                );
            }
        }
    }

    if command.is_cleaning() {
        let recipes = if preloaded_recipes.is_empty() {
            CookRecipe::from_list(recipe_names)?
        } else {
            preloaded_recipes.into_values().collect()
        };

        // no need to load dependencies
        return Ok((config, command, recipes));
    }

    let lock = &get_config().recipe_lock;

    let mut recipes = {
        let repo_binary = override_filesystem_repo_binary;

        // Expand deps for "source" + "local" and "binary"
        // This is the complete map from filesystem config
        let mut source_names: Vec<PackageName> = Vec::new();
        let mut binary_names: Vec<PackageName> = Vec::new();
        let mut special_rules: HashMap<PackageName, String> = HashMap::new();
        let default_rule = if repo_binary { "binary" } else { "source" };

        for (recipe_name, recipe_lock) in lock.iter().filter(|(_, v)| v.fsrule.is_some()) {
            let Ok(recipe_name) = PackageName::new(recipe_name) else {
                continue;
            };
            let rule = recipe_lock.fsrule.as_ref().unwrap();
            special_rules.insert(recipe_name.clone(), rule.to_string());
            // lock rules does not recurse as it's done already in the file
        }
        if let Some(conf) = config.filesystem.as_ref() {
            for (recipe_name_str, recipe_config) in conf.packages.iter() {
                let Ok(recipe_name) = PackageName::new(recipe_name_str) else {
                    continue;
                };

                let rule = if let Some(rule) = special_rules.get(&recipe_name) {
                    rule.as_str()
                } else if let PackageConfig::Build(rule) = recipe_config {
                    special_rules.insert(recipe_name.clone(), rule.to_string());
                    rule
                } else {
                    default_rule
                };

                if rule == "source" || rule == "local" {
                    source_names.push(recipe_name);
                } else if rule == "binary" {
                    binary_names.push(recipe_name);
                }
            }
        }
        source_names = CookRecipe::get_all_deps_names_recursive(&source_names, true)?;
        binary_names = CookRecipe::get_all_deps_names_recursive(&binary_names, false)?;
        let source_names: HashSet<PackageName> = source_names.into_iter().collect();
        let binary_names: HashSet<PackageName> = binary_names.into_iter().collect();

        // These are list that derived from recipe_names
        let mut source_recipe_names: Vec<PackageName> = Vec::new();
        let mut binary_recipe_names: Vec<PackageName> = Vec::new();
        let mut ignore_recipe_names: Vec<PackageName> = Vec::new();
        for recipe_name in recipe_names.iter() {
            if source_names.contains(recipe_name) {
                source_recipe_names.push(recipe_name.clone());
            } else if binary_names.contains(recipe_name) {
                binary_recipe_names.push(recipe_name.clone());
            } else {
                if special_rules
                    .get(recipe_name)
                    .is_some_and(|s| s == "ignore")
                {
                    ignore_recipe_names.push(recipe_name.clone());
                } else if repo_binary {
                    binary_recipe_names.push(recipe_name.clone());
                } else {
                    source_recipe_names.push(recipe_name.clone());
                }
            }
        }

        if config.with_package_deps || command.is_pushing() {
            source_recipe_names =
                CookRecipe::get_package_deps_recursive(&source_recipe_names, true)?;
            binary_recipe_names =
                CookRecipe::get_package_deps_recursive(&binary_recipe_names, true)?;
        }

        let mut recipes = if matches!(config.all, Some(AllOption::AllBinaries)) {
            // Removes all source-compiled recipes
            Vec::new()
        } else if command.is_building() || command.is_pushing() {
            // Pushing do not need dev deps, so does binary recipes at building
            let include_dev = command.is_building();
            if include_dev && default_rule == "source" {
                // let's cover a very specific case, binary -> source -> binary -> dev
                // in this case, we need to move that "source" to "binary", because
                // that would include dev from its binary child, which is unnecessary
                let mut i = 0;
                while i < source_recipe_names.len() {
                    let name = &source_recipe_names[i];
                    match special_rules.get(name) {
                        Some(s) if s.as_str() == "source" && binary_names.contains(name) => {
                            let bin = source_recipe_names.remove(i);
                            binary_recipe_names.push(bin);
                            continue;
                        }
                        _ => {}
                    }
                    i += 1;
                }
            }
            CookRecipe::get_build_deps_recursive(&source_recipe_names, include_dev)?
        } else {
            CookRecipe::from_list(source_recipe_names.clone())?
        };

        let binary_recipes = if command.is_building() || command.is_pushing() {
            CookRecipe::get_build_deps_recursive(&binary_recipe_names, false)?
        } else {
            CookRecipe::from_list(binary_recipe_names.clone())?
        };

        let ignore_recipes = CookRecipe::from_list(ignore_recipe_names.clone())?;

        recipes.extend(binary_recipes);
        recipes.extend(ignore_recipes);
        recipes = recipes_flatten_package_names(recipes);

        for recipe in recipes.iter_mut() {
            if let Some(special_rule) =
                special_rules.get(recipe.canon_recipe_name().without_prefix())
            {
                if recipe.name.is_host() && special_rule == "binary" {
                    // host recipe binaries is currently not supported
                    continue;
                }
                recipe.apply_filesystem_config(special_rule)?;
                continue;
            }
            let rule = match (
                source_names.contains(&recipe.name),
                binary_names.contains(&recipe.name),
            ) {
                (true, true) => {
                    // both lists: flip logic
                    if repo_binary { "source" } else { "binary" }
                }
                (true, false) => "source",
                (false, true) => "binary",
                (false, false) => default_rule,
            };
            if recipe.name.is_host() && rule == "binary" {
                // host recipe binaries is currently not supported
                continue;
            }

            recipe.apply_filesystem_config(rule)?;
        }

        recipes
    };

    if !get_config().recipe_lock.is_empty() {
        for recipe in recipes.iter_mut() {
            if let Some(gitrev) = lock
                .get(recipe.name.as_str())
                .and_then(|r| r.gitrev.clone())
            {
                if let Some(SourceRecipe::Git { rev, branch, .. }) = &mut recipe.recipe.source {
                    *rev = Some(gitrev.clone());
                    *branch = None;
                } else {
                    println!(
                        "DEBUG: Recipe {:?} contains into git rev but recipe source is not git",
                        recipe.name.as_str()
                    );
                }
                recipe.pinned = true;
            }
        }
    }

    if command.is_building() && recipes.iter().any(|r| r.rule == "binary") {
        let (_, repository) = fetch_repo::get_binary_repo();
        for recipe in recipes.iter_mut() {
            if recipe.rule == "binary" && !repository.packages.contains_key(recipe.name.as_str()) {
                if config.cook.verbose && !(config.cook.tui && command == CliCommand::Cook) {
                    // TODO: this should be printed at fetch log, not here
                    println!(
                        "DEBUG: Recipe {:?} has no binary package",
                        recipe.name.as_str()
                    );
                }
                recipe.rule = "source".into();
                recipe.reload_recipe()?;
            }
        }
    }

    if !config.with_package_deps || command.is_informational() {
        // In CliCommand::Cook, is_deps==true will make it skip checking source
        recipes_mark_as_deps(&recipe_names, &mut recipes);
    }

    Ok((config, command, recipes))
}

fn handle_fetch(
    recipe: &CookRecipe,
    config: &CliConfig,
    allow_offline: bool,
    logger: &PtyOut,
) -> Result<FetchResult> {
    match config.cook.offline && allow_offline {
        true => fetch_offline(recipe, logger),
        false => fetch(recipe, !recipe.is_deps, logger),
    }
}

fn handle_cook(
    recipe: &CookRecipe,
    config: &CliConfig,
    source_dir: PathBuf,
    logger: &PtyOut,
) -> Result<bool> {
    let recipe_dir = &recipe.dir;
    let target_dir = create_target_dir(recipe_dir, recipe.target)?;
    let build_result = build(
        recipe_dir,
        &source_dir,
        &target_dir,
        recipe,
        &config.cook,
        logger,
    )?;

    package(recipe, &build_result, &config.cook, logger)?;

    if config.cook.clean_target || config.cook.write_filetree {
        for stage_dir in &build_result.stage_dirs {
            if stage_dir.is_dir() {
                if config.cook.write_filetree {
                    let mut stage_files_buf = Vec::new();
                    tree::walk_file_tree(stage_dir, "", &mut stage_files_buf)
                        .map_err(|e| Error::from_io_error(e, "Walking files tree"))?;
                    stage_files_buf.push("".into()); // trailing eol
                    fs::write(
                        stage_dir.with_added_extension("files"),
                        stage_files_buf.join("\n"),
                    )
                    .map_err(|e| Error::from_io_error(e, "Writing files tree"))?;
                }
                if config.cook.clean_target {
                    remove_all(stage_dir)?;
                }
            }
        }
    }
    Ok(build_result.cached)
}

/// delete stage artifacts upon nonstop failure to let repo_builder know
fn handle_nonstop_fail(recipe: &CookRecipe) -> cookbook::Result<()> {
    let target_dir = recipe.target_dir();
    let stage_dirs = get_stage_dirs(&recipe.recipe.optional_packages, &target_dir);
    for stage_dir in stage_dirs {
        remove_stage_dir(&stage_dir)?;
    }
    Ok(())
}

fn handle_clean(recipe: &CookRecipe, config: &CliConfig, command: &CliCommand) -> Result<bool> {
    let mut dir = recipe.dir.join("target");
    let mut cached = true;
    let unfetch = matches!(*command, CliCommand::Unfetch);
    let mut clean_target = true;
    if matches!(*command, CliCommand::CleanTarget) {
        dir = dir.join(redoxer::target())
    }
    if unfetch && recipe.rule == "binary" {
        // may contains downloaded binaries
        clean_target = false;
    }
    if clean_target && dir.exists() {
        remove_all(&dir)?;
        cached = false;
    }
    if unfetch {
        let dir = recipe.dir.join("source");
        if dir.exists() {
            remove_all(&dir)?;
            cached = false;
        }
        let tar = recipe.dir.join("source.tar");
        // remove tar if there's no blake3 or `make distclean`
        if tar.is_file() {
            if let Some(SourceRecipe::Tar { blake3, .. }) = &recipe.recipe.source
                && blake3.is_none()
            {
                remove_all(&tar)?;
                cached = false;
            } else if config.all.is_some() {
                remove_all(&tar)?;
                cached = false;
            }
        }
    }
    Ok(cached)
}

static PUSH_CONFIG: OnceLock<CliConfig> = OnceLock::new();
fn handle_push(recipes: &Vec<CookRecipe>, config: &CliConfig) -> Result<()> {
    if !config.sysroot_dir.is_dir() {
        return Err(Error::Other(format!(
            "{} is not exist. Please run `make mount` first.",
            config.sysroot_dir.display()
        )));
    }
    let recipe_map: HashMap<&PackageName, &CookRecipe> =
        recipes.iter().map(|r| (&r.name, r)).collect();
    PUSH_CONFIG
        .set(config.clone())
        .unwrap_or_else(|_| panic!("PUSH_CONFIG is initialized"));
    let handle_push_inner = move |item: TreeItem| -> Result<bool> {
        let package_name = &item.recipe.name;
        if package_name.is_host() {
            return Ok(true); // TODO: skip altogether from recipes list
        }
        let r = match item.entry {
            WalkTreeEntry::Built(_) => {
                let config = PUSH_CONFIG.get().unwrap();
                let install_path = &config.sysroot_dir;
                let archive_path = item.recipe.stage_paths().1;
                let mut state = if !config.no_metadata {
                    Some(PackageState::from_sysroot(install_path).map_err(Error::from)?)
                } else {
                    None
                };
                let r = package_handle_push(state.as_mut(), &archive_path, install_path);
                if matches!(r, Ok(false)) && state.is_some() {
                    state
                        .unwrap()
                        .to_sysroot(install_path)
                        .map_err(|e| Error::from_io_error(e, "Extracting package"))?;
                }
                r
            }
            WalkTreeEntry::NotBuilt => Err(Error::Other(format!(
                "Package {} has not been built",
                package_name.name()
            ))),
            WalkTreeEntry::Deduped | WalkTreeEntry::Missing => {
                // does not matter
                return Ok(false);
            }
        };
        match r {
            Ok(true) => {
                print_cached(&CliCommand::Push, package_name);
                Ok(true)
            }
            Ok(false) => {
                print_success(&CliCommand::Push, package_name);
                Ok(false)
            }
            Err(e) => {
                print_failed(&CliCommand::Push, package_name);
                if get_config().cook.nonstop {
                    Ok(true)
                } else {
                    Err(e)
                }
            }
        }
    };

    let mut data = TreeData::new();
    for recipe in recipes.iter() {
        tree::walk_tree_entry(
            &recipe.name,
            &recipe_map,
            None,
            "",
            false, // don't care
            TreeOptions::Push,
            &mut data,
            handle_push_inner,
        )?;
    }

    if config.cook.verbose {
        println!();
        println!(
            "Pushed {} of {} {}",
            tree::format_size(data.total_size),
            data.total_count,
            if data.total_count == 1 {
                "package"
            } else {
                "packages"
            },
        );
    }

    Ok(())
}

fn handle_tree(recipes: &Vec<CookRecipe>, cmd: TreeOptions, config: &CliConfig) -> Result<()> {
    let recipe_map: HashMap<&PackageName, &CookRecipe> =
        recipes.iter().map(|r| (&r.name, r)).collect();
    let roots: Vec<PackageName> = recipes.iter().map(|s| s.name.clone()).collect();
    let data = tree::display_tree_entry(&roots[..], &recipe_map, cmd, config.display)?;

    if matches!(config.display, DisplayOptions::Tree) {
        println!();
        match cmd {
            TreeOptions::Repo => {}
            TreeOptions::Cook => println!(
                "Build summary: {} need build with total of {} {}",
                data.total_notbuilt,
                data.visited.len(),
                if data.visited.len() == 1 {
                    "recipe"
                } else {
                    "recipes"
                },
            ),
            TreeOptions::Push => println!(
                "Estimated image size: {} of {} {}",
                tree::format_size(data.total_size),
                data.visited.len(),
                if data.visited.len() == 1 {
                    "package"
                } else {
                    "packages"
                },
            ),
        }
    }

    Ok(())
}

fn handle_change_rule(
    recipes: &Vec<CookRecipe>,
    config: &CliConfig,
    command: &CliCommand,
) -> Result<()> {
    let mut lock = get_config().recipe_lock.clone();
    let cookbook_date = get_git_commit_date(&PathBuf::from("."))?;
    let is_change_rule = matches!(
        command,
        CliCommand::ChangeRule | CliCommand::ChangeRuleLocal
    );
    let is_capture_rev = matches!(command, CliCommand::CaptureRev);
    for recipe in recipes {
        if is_change_rule && recipe.name.is_host() {
            // host packages will always be "source" so it's pointless to change their rule
            continue;
        }
        if is_capture_rev && !matches!(recipe.recipe.source, Some(SourceRecipe::Git { .. })) {
            continue;
        }
        let recipe_name = recipe.canon_recipe_name();
        let recipe_name = recipe_name.without_prefix();
        let mut recipe_lock = lock.get(recipe_name).cloned().unwrap_or_default();
        let cached = if is_change_rule {
            if config.unset {
                recipe_lock.fsrule.take().is_none()
            } else {
                let new_rule = config
                    .set_rule
                    .as_ref()
                    .cloned()
                    .unwrap_or_else(|| recipe.rule.clone());

                let old_rule = recipe_lock.fsrule.replace(new_rule.clone());
                old_rule == Some(new_rule)
            }
        } else if is_capture_rev {
            if config.unset {
                recipe_lock.gitrev.take().is_none()
            } else {
                let source_dir = recipe.dir.join("source");
                let rev = if config.with_rollback {
                    // invoke fetch as the git tracking can be different
                    match handle_fetch(recipe, config, false, &None) {
                        Ok(_) => get_git_rev_before_date(&source_dir, &cookbook_date),
                        Err(e) => Err(e),
                    }
                } else {
                    get_git_head_rev(&source_dir).map(|r| r.0)
                };
                match rev {
                    Ok(rev) => {
                        let old_rev = recipe_lock.gitrev.replace(rev.clone());
                        old_rev == Some(rev)
                    }
                    Err(e) => {
                        eprintln!("Skipping {}: {e}", recipe.name.as_str());
                        continue;
                    }
                }
            }
        } else {
            unreachable!()
        };
        if recipe_lock.is_empty() {
            lock.remove(recipe_name);
        } else {
            lock.insert(recipe_name.to_string(), recipe_lock);
        }
        let clean_cached = if !cached && is_change_rule {
            handle_clean(recipe, config, &CliCommand::Clean)?
        } else {
            true
        };

        if cached && clean_cached {
            print_cached(command, &recipe.name);
        } else {
            print_success(command, &recipe.name);
        }
    }
    CookLockOpt { recipes: lock }.save();
    Ok(())
}

macro_rules! bail_options_err {
    ($($arg:tt)*) => {
        return Err(cookbook::Error::Options(format!($($arg)*)))
    };
}

use bail_options_err;
