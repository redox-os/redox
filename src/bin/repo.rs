use ansi_to_tui::IntoText;
use anyhow::{Context, anyhow, bail};
use cookbook::config::{CookConfig, get_config, init_config};
use cookbook::cook::cook_build::build;
use cookbook::cook::fetch::{fetch, fetch_offline};
use cookbook::cook::fs::{create_target_dir, run_command};
use cookbook::cook::package::package;
use cookbook::cook::pty::{PtyOut, UnixSlavePty, flush_pty, setup_pty};
use cookbook::cook::script::KILL_ALL_PID;
use cookbook::cook::tree::{WalkTreeEntry, display_tree_entry, format_size, walk_tree_entry};
use cookbook::log_to_pty;
use cookbook::recipe::CookRecipe;
use pkg::PackageName;
use pkg::package::PackageError;
use ratatui::Terminal;
use ratatui::layout::{Constraint, Direction, Layout, Position, Rect};
use ratatui::prelude::TermionBackend;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use redox_installer::PackageConfig;
use redoxer::target;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{Read, Write, stderr, stdin, stdout};
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, OnceLock, mpsc};
use std::time::{Duration, Instant};
use std::{cmp, env, fs};
use std::{process, thread};
use termion::event::{Event, Key, MouseEvent};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::IntoAlternateScreen;
use termion::{color, style};

// A repo manager, to replace repo.sh

const REPO_HELP_STR: &str = r#"
    Usage: repo <command> [flags] <recipe1> <recipe2> ...

    command list:
        fetch     download recipe sources
        cook      build recipe packages
        unfetch   delete recipe sources
        clean     delete recipe artifacts
        push      extract package into sysroot
        find      find path of recipe packages
        tree      show tree of recipe packages
    
    common flags:
        --cookbook=<cookbook_dir>  the "recipes" folder, default to $PWD/recipes
        --repo=<repo_dir>          the "repo" folder, default to $PWD/repo
        --sysroot=<sysroot_dir>    the "root" folder used for "push" command
            For Redox, defaults to "/", else default to $PWD/sysroot
        --with-package-deps        include package deps
        --all                      apply to all recipes in <cookbook_dir>
        --category=<category>      apply to all recipes in <cookbook_dir>/<category>
        --filesystem=<filesystem>  override recipes config using installer file

    cook env and their defaults:
        CI=                        set to any value to disable TUI
        COOKBOOK_LOGS=             whether to capture build logs (default is !CI)
        COOKBOOK_OFFLINE=false     prevent internet access if possible
        COOKBOOK_NONSTOP=false     pkeep running even a recipe build failed
        COOKBOOK_VERBOSE=true      print success/error on each recipe
        COOKBOOK_MAKE_JOBS=        override build jobs count from nproc
"#;

#[derive(Clone)]
struct CliConfig {
    cookbook_dir: PathBuf,
    repo_dir: PathBuf,
    sysroot_dir: PathBuf,
    logs_dir: Option<PathBuf>,
    category: Option<PathBuf>,
    filesystem: Option<redox_installer::Config>,
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
    Tree,
    Find,
}

impl CliCommand {
    pub fn is_informational(&self) -> bool {
        *self == CliCommand::Tree || *self == CliCommand::Find
    }
    pub fn is_building(&self) -> bool {
        *self == CliCommand::Fetch || *self == CliCommand::Cook
    }
    pub fn is_pushing(&self) -> bool {
        *self == CliCommand::Push || *self == CliCommand::Tree
    }
    pub fn is_cleaning(&self) -> bool {
        *self == CliCommand::Clean || *self == CliCommand::Unfetch
    }
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
            "tree" => Ok(CliCommand::Tree),
            "find" => Ok(CliCommand::Find),
            _ => Err(anyhow!("Unknown command '{}'\n{}\n", s, REPO_HELP_STR)),
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
            CliCommand::Tree => "tree".to_string(),
            CliCommand::Find => "find".to_string(),
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
            // build dir here is hardcoded in repo_builder as well
            logs_dir: if get_config().cook.logs {
                Some(current_dir.join("build/logs"))
            } else {
                None
            },
            category: None,
            sysroot_dir: if cfg!(target_os = "redox") {
                PathBuf::from("/")
            } else {
                current_dir.join("sysroot")
            },
            with_package_deps: false,
            cook: get_config().cook.clone(),
            all: false,
            filesystem: None,
        })
    }
}

fn main() {
    init_config();
    if let Err(e) = main_inner() {
        eprintln!("{:?}", e);
        process::exit(1);
    };
}

fn main_inner() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        println!("{}", REPO_HELP_STR);
        process::exit(1);
    }

    let (config, command, recipe_names) = parse_args(args)?;
    if command == CliCommand::Cook && config.cook.tui {
        if let Some((name, e)) = run_tui_cook(config.clone(), recipe_names.clone())? {
            let _ = stderr().write(e.as_bytes());
            let _ = stderr().write(b"\n\n");
            print_failed(&command, &name);
            return Err(anyhow!("Execution has failed"));
        } else {
            print_success(&command, None);
        }
        return publish_packages(&recipe_names, &config.repo_dir);
    }
    if command == CliCommand::Tree {
        return handle_tree(&recipe_names, &config);
    }
    if command == CliCommand::Push {
        return handle_push(&recipe_names, &config);
    }

    let verbose = config.cook.verbose;
    for recipe in &recipe_names {
        match repo_inner(&config, &command, recipe) {
            Ok(_) => {
                print_success(&command, Some(&recipe.name));
            }
            Err(e) => {
                if config.cook.nonstop && verbose {
                    eprintln!("{:?}", e);
                }
                print_failed(&command, &recipe.name);
                if !config.cook.nonstop {
                    return Err(e);
                }
            }
        }
    }

    if command == CliCommand::Cook {
        return publish_packages(&recipe_names, &config.repo_dir);
    }

    if verbose {
        println!(
            "\nCommand '{}' completed for {} recipes.",
            command.to_string(),
            recipe_names.len()
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

fn print_success(command: &CliCommand, recipe: Option<&PackageName>) {
    if let Some(recipe) = recipe {
        eprintln!(
            "{}{}{} {} - successful{}{}",
            style::Bold,
            color::Fg(color::AnsiValue(46)),
            command.to_string(),
            recipe.as_str(),
            color::Fg(color::Reset),
            style::Reset,
        );
    } else {
        eprintln!(
            "{}{}{} - successful{}{}",
            style::Bold,
            color::Fg(color::AnsiValue(46)),
            command.to_string(),
            color::Fg(color::Reset),
            style::Reset,
        );
    }
}

fn repo_inner(
    config: &CliConfig,
    command: &CliCommand,
    recipe: &CookRecipe,
) -> Result<(), anyhow::Error> {
    Ok(match *command {
        CliCommand::Fetch | CliCommand::Cook => {
            let repo_inner_fn = move |logger: &PtyOut| -> Result<(), anyhow::Error> {
                let source_dir = handle_fetch(recipe, config, logger)?;
                if *command == CliCommand::Cook {
                    handle_cook(recipe, config, source_dir, recipe.is_deps, logger)?;
                }
                Ok(())
            };
            let Some(log_path) = &config.logs_dir else {
                return repo_inner_fn(&None);
            };

            let (status_tx, status_rx) = mpsc::channel::<StatusUpdate>();
            let (mut stdout_writer, mut stderr_writer) = setup_logger(&status_tx, &recipe.name);
            let mut app = TuiApp::new(vec![recipe.clone()]);
            app.dump_logs_anyway = true;
            let th = thread::spawn(move || {
                while let Ok(update) = status_rx.recv() {
                    if update == StatusUpdate::CookThreadFinished {
                        break;
                    }
                    app.update_status(update);
                }
            });
            let mut logger = Some((&mut stdout_writer, &mut stderr_writer));
            let result = repo_inner_fn(&logger);
            if let Err(err_ctx) = &result {
                log_to_pty!(&logger, "\n{:?}", err_ctx)
            }
            // successful fetch is not that useful to log
            if *command == CliCommand::Cook || result.is_err() {
                flush_pty(&mut logger);
                let log_path = log_path.join(format!("{}.log", recipe.name.as_str()));
                status_tx
                    .send(StatusUpdate::FlushLog(recipe.name.clone(), log_path))
                    .unwrap_or_default();
            }
            status_tx
                .send(StatusUpdate::CookThreadFinished)
                .unwrap_or_default();
            let _ = th.join();
        }
        CliCommand::Unfetch => handle_clean(recipe, config, true, true)?,
        CliCommand::Clean => handle_clean(recipe, config, false, true)?,
        CliCommand::Push => unreachable!(),
        CliCommand::Tree => unreachable!(),
        CliCommand::Find => println!("{}", recipe.dir.display()),
    })
}

fn publish_packages(recipe_names: &Vec<CookRecipe>, repo_path: &PathBuf) -> anyhow::Result<()> {
    let repo_bin = env::current_exe()?.parent().unwrap().join("repo_builder");
    let mut command = Command::new(repo_bin);
    command
        .arg(repo_path.join(redoxer::target()))
        .args(recipe_names.iter().filter_map(|n| {
            if !n.is_deps {
                Some(n.name.as_str())
            } else {
                None
            }
        }));

    run_command(command, &None).map_err(|e| anyhow!(e))
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
                    "--category" => config.category = Some(PathBuf::from(value)),
                    "--filesystem" => {
                        config.filesystem = Some({
                            let r = redox_installer::Config::from_file(&PathBuf::from(value));
                            r.context("Unable to read filesystem installer config")?
                        })
                    }
                    _ => {
                        eprintln!("Error: Unknown flag with value: {}", arg);
                        process::exit(1);
                    }
                }
            } else if arg.starts_with("--category-") {
                // to workaround make command limit we provide this option
                config.category = Some(PathBuf::from(arg[("--category-").len()..].to_owned()));
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

    if let Some(c) = config.category {
        // need to prefix by cookbook dir
        config.category = Some(PathBuf::from("recipes").join(c));
    }
    if let Some(c) = config.logs_dir.as_mut() {
        *c = c.join(redoxer::target());
        fs::create_dir_all(c).map_err(|e| anyhow!(e))?;
    }

    let command = command.ok_or(anyhow!("Error: No command specified."))?;
    let command: CliCommand = str::parse(&command)?;
    let mut recipes = if config.all || config.category.is_some() {
        if !recipe_names.is_empty() {
            bail!("Do not specify recipe names when using the --all or --category flag.");
        }
        if config.all && config.category.is_some() {
            bail!("Do not specify both --all and --category flag.");
        }
        if config.all && !command.is_cleaning() {
            // because read_recipe is false by logic below
            // some recipes on wip folders are invalid anyway
            bail!(
                "Refusing to run an unrealistic command to {} all recipes",
                command.to_string()
            );
        }
        match &config.category {
            None => pkg::recipes::list(""),
            Some(prefix) => pkg::recipes::list("")
                .into_iter()
                .filter(|p| p.starts_with(prefix))
                .collect(),
        }
        .iter()
        .map(|f| CookRecipe::from_path(f, !config.all))
        .collect::<Result<Vec<CookRecipe>, PackageError>>()?
    } else {
        if recipe_names.is_empty() {
            if let Some(conf) = config.filesystem.as_ref() {
                recipe_names = conf
                    .packages
                    .iter()
                    .filter_map(|(f, v)| {
                        match v {
                            PackageConfig::Build(rule) if rule == "ignore" => {
                                return None;
                            }
                            _ => {}
                        }
                        PackageName::new(f).ok()
                    })
                    .collect();
            } else {
                bail!(
                    "Error: No recipe names or filesystem config provided and --all flag was not used."
                );
            }
        }
        if command.is_building() || (command.is_pushing() && config.with_package_deps) {
            if config.with_package_deps {
                recipe_names = CookRecipe::get_package_deps_recursive(&recipe_names, true)
                    .context("failed get package deps")?;
            }
            CookRecipe::get_build_deps_recursive(
                &recipe_names,
                true,
                // In CliCommand::Cook, is_deps==true will make it skip checking source
                command == CliCommand::Tree || !config.with_package_deps,
            )?
        } else {
            recipe_names
                .iter()
                .map(|f| CookRecipe::from_name(f.as_str()).unwrap())
                .collect()
        }
    };
    if let Some(conf) = config.filesystem.as_ref()
        && !command.is_cleaning()
    {
        for recipe in recipes.iter_mut() {
            if let Some(recipe_conf) = conf.packages.get(recipe.name.as_str()) {
                match recipe_conf {
                    // build from source as usual
                    PackageConfig::Build(rule) if rule == "source" => {}
                    // keep local changes
                    PackageConfig::Build(rule) if rule == "local" => recipe.recipe.source = None,
                    // download from remote build
                    PackageConfig::Build(rule) if rule == "binary" => {
                        recipe.recipe.source = None;
                        recipe.recipe.build.set_as_remote();
                    }
                    // don't build this recipe (unlikely to go here unless some deps need it)
                    // TODO: Note that we're assuming this being ignored from e.g. metapackages
                    // TODO: Will totally broke build if this recipe needed as some other build dependencies
                    PackageConfig::Build(rule) if rule == "ignore" => {
                        recipe.recipe.source = None;
                        recipe.recipe.build.set_as_none();
                    }
                    PackageConfig::Build(rule) => {
                        bail!(
                            // Fail fast because we could risk losing local changes if "local" was typo'ed
                            "Invalid pkg config {} = \"{}\"\nExpecting either 'source', 'local', 'binary' or 'ignore'",
                            recipe.name.as_str(),
                            rule
                        );
                    }
                    _ => {
                        if conf.general.repo_binary == Some(true) {
                            // same reason as Build("binary")
                            recipe.recipe.source = None;
                            recipe.recipe.build.set_as_remote();
                        }
                    }
                }
            } else {
                if conf.general.repo_binary == Some(true) {
                    recipe.recipe.source = None;
                    recipe.recipe.build.set_as_remote();
                }
            }
        }
    }

    if command.is_informational() {
        // avoid extra data that clobber stdout
        config.cook.verbose = false;
    }

    Ok((config, command, recipes))
}

fn handle_fetch(
    recipe: &CookRecipe,
    config: &CliConfig,
    logger: &PtyOut,
) -> anyhow::Result<PathBuf> {
    let recipe_dir = &recipe.dir;
    let source_dir = match config.cook.offline {
        true => fetch_offline(recipe_dir, &recipe.recipe, logger),
        false => fetch(recipe_dir, &recipe.recipe, logger),
    }
    .map_err(|e| anyhow!("failed to fetch: {:?}", e))?;

    Ok(source_dir)
}

fn handle_cook(
    recipe: &CookRecipe,
    config: &CliConfig,
    source_dir: PathBuf,
    is_deps: bool,
    logger: &PtyOut,
) -> anyhow::Result<()> {
    let recipe_dir = &recipe.dir;
    let target_dir = create_target_dir(recipe_dir).map_err(|e| anyhow!(e))?;
    let (stage_dir, auto_deps) = build(
        recipe_dir,
        &source_dir,
        &target_dir,
        &recipe.name,
        &recipe.recipe,
        config.cook.offline,
        !is_deps,
        logger,
    )
    .map_err(|err| anyhow!("failed to build: {:?}", err))?;

    package(
        &stage_dir,
        &target_dir,
        &recipe.name,
        &recipe.recipe,
        &auto_deps,
        logger,
    )
    .map_err(|err| anyhow!("failed to package: {:?}", err))?;

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

static PUSH_SYSROOT_DIR: OnceLock<PathBuf> = OnceLock::new();
fn handle_push(recipes: &Vec<CookRecipe>, config: &CliConfig) -> anyhow::Result<()> {
    let recipe_map: HashMap<&PackageName, &CookRecipe> =
        recipes.iter().map(|r| (&r.name, r)).collect();
    let mut total_size: u64 = 0;
    let mut visited: HashSet<PackageName> = HashSet::new();
    let roots: Vec<&CookRecipe> = recipes.iter().filter(|r| !r.is_deps).collect();
    let num_roots = roots.len();
    PUSH_SYSROOT_DIR.set(config.sysroot_dir.clone()).unwrap();
    let handle_push_inner = move |package_name: &PackageName,
                                  _prefix: &str,
                                  _is_last: bool,
                                  entry: &WalkTreeEntry|
          -> anyhow::Result<()> {
        let public_path = "build/id_ed25519.pub.toml";
        let r = match entry {
            WalkTreeEntry::Built(archive_path, _) => {
                let sysroot_dir = PUSH_SYSROOT_DIR.get().unwrap();
                pkgar::extract(public_path, archive_path.as_path(), sysroot_dir).context(format!(
                    "failed to install '{}' in '{}'",
                    archive_path.display(),
                    sysroot_dir.display(),
                ))
            }
            WalkTreeEntry::NotBuilt => Err(anyhow!(
                "Package {} has not been built",
                package_name.as_str()
            )),
            WalkTreeEntry::Deduped | WalkTreeEntry::Missing => {
                return Ok(());
            }
        };
        match r {
            Ok(()) => {
                print_success(&CliCommand::Push, Some(package_name));
                Ok(())
            }
            Err(e) => {
                print_failed(&CliCommand::Push, package_name);
                if get_config().cook.nonstop {
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    };
    if config.with_package_deps {
        for (i, root) in roots.iter().enumerate() {
            walk_tree_entry(
                &root.name,
                &recipe_map,
                "",
                i == num_roots - 1,
                &mut visited,
                &mut total_size,
                handle_push_inner,
            )?;
        }
    } else {
        for (i, root) in roots.iter().enumerate() {
            let archive_path = config
                .repo_dir
                .join(target())
                .join(format!("{}.pkgar", root.name));
            let metadata = std::fs::metadata(&archive_path);
            handle_push_inner(
                &root.name,
                "",
                i == num_roots - 1,
                &match metadata {
                    Ok(m) => {
                        total_size += m.len();
                        visited.insert(root.name.clone());
                        WalkTreeEntry::Built(&archive_path, m.len())
                    }
                    Err(_) => WalkTreeEntry::NotBuilt,
                },
            )?;
        }
    }

    if config.cook.verbose {
        println!("");
        println!(
            "Pushed {} of {} {}",
            format_size(total_size),
            visited.len(),
            if visited.len() == 1 {
                "package"
            } else {
                "packages"
            },
        );
    }

    Ok(())
}

fn handle_tree(recipes: &Vec<CookRecipe>, _config: &CliConfig) -> anyhow::Result<()> {
    let recipe_map: HashMap<&PackageName, &CookRecipe> =
        recipes.iter().map(|r| (&r.name, r)).collect();
    let mut total_size: u64 = 0;
    let mut visited: HashSet<PackageName> = HashSet::new();
    let roots: Vec<&CookRecipe> = recipes.iter().filter(|r| !r.is_deps).collect();
    let num_roots = roots.len();
    for (i, root) in roots.iter().enumerate() {
        display_tree_entry(
            &root.name,
            &recipe_map,
            "",
            i == num_roots - 1,
            &mut visited,
            &mut total_size,
        )?;
    }

    println!("");
    println!(
        "Estimated image size: {} of {} {}",
        format_size(total_size),
        visited.len(),
        if visited.len() == 1 {
            "package"
        } else {
            "packages"
        },
    );

    Ok(())
}

//
// ------------- TUI SPECIFIC CODE -------------------
//

#[derive(Debug, Clone, PartialEq)]
enum RecipeStatus {
    Pending,
    Fetching,
    Fetched,
    Cooking,
    Done,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq)]
enum StatusUpdate {
    StartFetch(PackageName),
    Fetched(CookRecipe),
    FailFetch(CookRecipe, String),
    StartCook(PackageName),
    Cooked(CookRecipe),
    FailCook(CookRecipe, String),
    PushLog(PackageName, Vec<u8>),
    FlushLog(PackageName, PathBuf),
    FetchThreadFinished,
    CookThreadFinished,
}

#[derive(PartialEq)]
enum JobType {
    Fetch,
    Cook,
}

impl ToString for JobType {
    fn to_string(&self) -> String {
        match self {
            JobType::Fetch => "Fetch",
            JobType::Cook => "Cook",
        }
        .to_string()
    }
}

struct TuiApp {
    recipes: Vec<(CookRecipe, RecipeStatus)>,
    fetch_queue: VecDeque<CookRecipe>,
    cook_queue: VecDeque<CookRecipe>,
    done: Vec<PackageName>,
    active_fetch: Option<PackageName>,
    active_cook: Option<PackageName>,
    logs: HashMap<PackageName, Vec<String>>,
    log_byte_buffer: HashMap<PackageName, Vec<u8>>,
    log_scroll: usize,
    log_view_job: JobType,
    auto_scroll: bool,
    fetch_scroll: usize,
    cook_scroll: usize,
    cook_auto_scroll: bool,
    cook_list_state: ListState,
    fetch_complete: bool,
    cook_complete: bool,
    fetch_panel_rect: Option<Rect>,
    cook_panel_rect: Option<Rect>,
    log_panel_rect: Option<Rect>,
    prompt: Option<FailurePrompt>,
    dump_logs_anyway: bool,
    dump_logs_on_exit: Option<(PackageName, String)>,
}

impl TuiApp {
    fn new(recipes: Vec<CookRecipe>) -> Self {
        Self {
            recipes: recipes
                .iter()
                .cloned()
                .map(|r| (r, RecipeStatus::Pending))
                .collect(),
            fetch_queue: recipes.iter().cloned().map(|r| r.clone()).collect(),
            cook_queue: VecDeque::new(),
            done: Vec::new(),
            active_fetch: None,
            active_cook: None,
            logs: HashMap::new(),
            log_byte_buffer: HashMap::new(),
            log_scroll: 0,
            auto_scroll: true,
            log_view_job: JobType::Fetch,
            fetch_scroll: 0,
            cook_scroll: 0,
            cook_auto_scroll: true,
            cook_list_state: ListState::default(),
            fetch_complete: false,
            cook_complete: false,
            fetch_panel_rect: None,
            cook_panel_rect: None,
            log_panel_rect: None,
            prompt: None,
            dump_logs_anyway: false,
            dump_logs_on_exit: None,
        }
    }

    pub fn get_active_name(&self) -> Option<PackageName> {
        if self.log_view_job == JobType::Cook {
            self.active_cook.clone()
        } else {
            self.active_fetch.clone()
        }
    }

    pub fn get_active_log(
        &self,
    ) -> (
        Option<PackageName>,
        Option<&Vec<String>>,
        Option<Cow<'_, str>>,
    ) {
        let active_name = self.get_active_name();
        let (log_text, log_line) = if let Some(active_name) = active_name.as_ref() {
            self.get_recipe_log(active_name)
        } else {
            (None, None)
        };

        (active_name, log_text, log_line)
    }

    pub fn get_recipe_log(
        &self,
        recipe_name: &PackageName,
    ) -> (Option<&Vec<String>>, Option<Cow<'_, str>>) {
        let log_text = self.logs.get(recipe_name);
        let log_line = if let Some(b) = self.log_byte_buffer.get(recipe_name) {
            Some(String::from_utf8_lossy(b))
        } else {
            None
        };
        (log_text, log_line)
    }

    pub fn write_log(&self, recipe_name: &PackageName, log_path: &PathBuf) -> anyhow::Result<()> {
        let (Some(logs), line) = self.get_recipe_log(recipe_name) else {
            return Ok(());
        };
        let str = strip_ansi_escapes::strip_str(join_logs(logs, line));
        if !str.trim_end().is_empty() {
            fs::write(log_path, str)?;
        }
        return Ok(());
    }

    // Update the state based on a message from a worker thread
    fn update_status(&mut self, update: StatusUpdate) {
        let (name, new_status) = match update {
            StatusUpdate::StartFetch(name) => {
                self.active_fetch = Some(name.clone());
                self.logs.insert(name.clone(), Vec::new());
                self.log_byte_buffer.insert(name.clone(), Vec::new());
                self.log_scroll = 0;
                self.auto_scroll = true;
                (name.clone(), RecipeStatus::Fetching)
            }
            StatusUpdate::Fetched(recipe) => (recipe.name.clone(), RecipeStatus::Fetched),
            StatusUpdate::FailFetch(recipe, err) => {
                self.prompt = Some(FailurePrompt::new(recipe.clone(), err.clone()));
                (recipe.name.clone(), RecipeStatus::Failed(err))
            }
            StatusUpdate::StartCook(name) => {
                self.active_cook = Some(name.clone());
                self.logs.insert(name.clone(), Vec::new());
                self.log_byte_buffer.insert(name.clone(), Vec::new());
                (name.clone(), RecipeStatus::Cooking)
            }
            StatusUpdate::PushLog(name, chunk) => {
                let buffer = self.log_byte_buffer.entry(name.clone()).or_default();
                buffer.extend_from_slice(&chunk);
                if self.dump_logs_anyway {
                    let _ = std::io::stdout().write_all(&chunk);
                }
                let log_list = self.logs.entry(name.clone()).or_default();
                while let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
                    let line_bytes = buffer.drain(..=newline_pos).collect::<Vec<u8>>();
                    let line_str = String::from_utf8_lossy(&line_bytes).into_owned();
                    let line_str_pos = line_str.trim_end();
                    let line_str = line_str_pos.rsplit('\r').next().unwrap_or(&line_str_pos);
                    log_list.push(line_str.to_owned());
                }
                return;
            }
            StatusUpdate::FlushLog(name, path) => {
                // TODO: This blocks the TUI, maybe open separate thread?
                // FIXME: handle error here?
                let _ = self.write_log(&name, &path);
                return;
            }
            StatusUpdate::Cooked(recipe) => {
                if self.active_cook.as_ref() == Some(&recipe.name) {
                    self.active_cook = None;
                }
                self.auto_scroll = true;
                (recipe.name.clone(), RecipeStatus::Done)
            }
            StatusUpdate::FailCook(recipe, err) => {
                self.prompt = Some(FailurePrompt::new(recipe.clone(), err.clone()));
                (recipe.name.clone(), RecipeStatus::Failed(err))
            }
            StatusUpdate::FetchThreadFinished => {
                self.fetch_complete = true;
                self.log_view_job = JobType::Cook;
                return;
            }
            StatusUpdate::CookThreadFinished => {
                self.cook_complete = true;
                return;
            }
        };

        if let Some((_, status)) = self.recipes.iter_mut().find(|(r, _)| r.name == name) {
            *status = new_status;
        }

        // Re-compute the queues for display
        self.fetch_queue = self
            .recipes
            .iter()
            .filter(|(_, s)| *s == RecipeStatus::Pending)
            .map(|(r, _)| r.clone())
            .collect();
        self.cook_queue = self
            .recipes
            .iter()
            .filter(|(_, s)| *s == RecipeStatus::Fetched)
            .map(|(r, _)| r.clone())
            .collect();
        self.done = self
            .recipes
            .iter()
            .filter(|(_, s)| *s == RecipeStatus::Done)
            .map(|(r, _)| r.name.clone())
            .collect();
    }
}

fn run_tui_cook(
    config: CliConfig,
    recipes: Vec<CookRecipe>,
) -> anyhow::Result<Option<(PackageName, String)>> {
    let (work_tx, work_rx) = mpsc::channel::<(CookRecipe, PathBuf)>();
    let (status_tx, status_rx) = mpsc::channel::<StatusUpdate>();

    let running = Arc::new(AtomicBool::new(true));
    let prompting = Arc::new(AtomicU32::new(0));
    const TICK_RATE: Duration = Duration::from_millis(100);

    // ---- Cooker Thread ----
    let cooker_config = config.clone();
    let cooker_status_tx = status_tx.clone();
    let cooker_prompting = prompting.clone();
    let cooker_handle = thread::spawn(move || {
        'done: for (recipe, source_dir) in work_rx {
            let name = recipe.name.clone();
            let is_deps = recipe.is_deps;
            let (mut stdout_writer, mut stderr_writer) = setup_logger(&cooker_status_tx, &name);
            let mut logger = Some((&mut stdout_writer, &mut stderr_writer));
            'again: loop {
                cooker_status_tx
                    .send(StatusUpdate::StartCook(name.clone()))
                    .unwrap();
                let handler = handle_cook(
                    &recipe,
                    &cooker_config,
                    source_dir.clone(),
                    is_deps,
                    &logger,
                );
                if let Some(log_path) = cooker_config.logs_dir.as_ref() {
                    if let Err(err_ctx) = &handler {
                        log_to_pty!(&logger, "\n{:?}", err_ctx)
                    }
                    flush_pty(&mut logger);
                    let log_path = log_path.join(format!("{}.log", name.as_str()));
                    cooker_status_tx
                        .send(StatusUpdate::FlushLog(name.clone(), log_path))
                        .unwrap_or_default();
                }
                match handler {
                    Ok(()) => {
                        cooker_status_tx
                            .send(StatusUpdate::Cooked(recipe))
                            .unwrap_or_default();
                        if cooker_config.cook.nonstop
                            && cooker_prompting.load(Ordering::SeqCst) == 4
                        {
                            break 'done;
                        }
                        break;
                    }
                    Err(e) => {
                        cooker_status_tx
                            .send(StatusUpdate::FailCook(recipe.clone(), e.to_string()))
                            .unwrap_or_default();
                        if cooker_config.cook.nonstop {
                            if cooker_prompting.load(Ordering::SeqCst) == 4 {
                                break 'done;
                            }
                            break;
                        }
                        while cooker_prompting.load(Ordering::SeqCst) != 0 {
                            thread::sleep(Duration::from_millis(101)); // wait other prompt
                        }
                        cooker_prompting.swap(1, Ordering::SeqCst);
                        'wait: loop {
                            match cooker_prompting.load(Ordering::SeqCst) {
                                0 => break 'again,
                                1 => thread::sleep(Duration::from_millis(101)),
                                2 => {
                                    cooker_prompting.swap(0, Ordering::SeqCst);
                                    break 'wait;
                                } // retry
                                3 => {
                                    cooker_prompting.swap(0, Ordering::SeqCst);
                                    break 'again;
                                } // skip
                                4 => {
                                    cooker_prompting.swap(0, Ordering::SeqCst);
                                    break 'done;
                                } // done
                                _ => unreachable!(),
                            }
                        }
                    }
                }
            }
        }
        cooker_status_tx
            .send(StatusUpdate::CookThreadFinished)
            .unwrap_or_default();
    });

    let mstdin = stdin();
    let mstdout = stdout()
        .into_raw_mode()
        .unwrap()
        .into_alternate_screen()
        .unwrap();

    // ----- Input Thread -----
    let (input_tx, input_rx) = mpsc::channel::<Event>();
    let _input_handle = thread::spawn(move || {
        for evt in mstdin.events() {
            if let Ok(evt) = evt {
                if input_tx.send(evt).is_err() {
                    return;
                }
            }
        }
    });

    // ---- Fetcher Thread ----
    let fetcher_recipes = recipes.clone();
    let fetcher_status_tx = status_tx.clone();
    let fetcher_config = config.clone();
    let fetcher_prompting = prompting.clone();
    let fetcher_handle = thread::spawn(move || {
        'done: for recipe in fetcher_recipes {
            let name = recipe.name.clone();
            let (mut stdout_writer, mut stderr_writer) = setup_logger(&fetcher_status_tx, &name);
            let mut logger = Some((&mut stdout_writer, &mut stderr_writer));
            'again: loop {
                fetcher_status_tx
                    .send(StatusUpdate::StartFetch(name.clone()))
                    .unwrap();
                let handler = handle_fetch(&recipe, &fetcher_config, &logger);
                if let Some(log_path) = fetcher_config.logs_dir.as_ref()
                    // successful fetch log usually not that helpful
                    && handler.is_err()
                {
                    if let Err(err_ctx) = &handler {
                        log_to_pty!(&logger, "\n{:?}", err_ctx)
                    }
                    flush_pty(&mut logger);
                    let log_path = log_path.join(format!("{}.log", name.as_str()));
                    fetcher_status_tx
                        .send(StatusUpdate::FlushLog(name.clone(), log_path))
                        .unwrap_or_default();
                }
                match handler {
                    Ok(source_dir) => {
                        fetcher_status_tx
                            .send(StatusUpdate::Fetched(recipe.clone()))
                            .unwrap();
                        if work_tx.send((recipe.clone(), source_dir)).is_err() {
                            // Cooker thread died
                            break 'done;
                        }
                        if fetcher_config.cook.nonstop
                            && fetcher_prompting.load(Ordering::SeqCst) == 4
                        {
                            break 'done;
                        }
                        break;
                    }
                    Err(e) => {
                        fetcher_status_tx
                            .send(StatusUpdate::FailFetch(recipe.clone(), e.to_string()))
                            .unwrap_or_default();
                        if fetcher_config.cook.nonstop {
                            if fetcher_prompting.load(Ordering::SeqCst) == 4 {
                                break 'done;
                            }
                            break;
                        }
                        while fetcher_prompting.load(Ordering::SeqCst) != 0 {
                            thread::sleep(Duration::from_millis(101)); // wait other prompt
                        }
                        fetcher_prompting.swap(1, Ordering::SeqCst);
                        'wait: loop {
                            match fetcher_prompting.load(Ordering::SeqCst) {
                                0 => break 'again,
                                1 => thread::sleep(Duration::from_millis(101)),
                                2 => {
                                    fetcher_prompting.swap(0, Ordering::SeqCst);
                                    break 'wait;
                                } // retry
                                3 => {
                                    fetcher_prompting.swap(0, Ordering::SeqCst);
                                    break 'again;
                                } // skip
                                4 => {
                                    fetcher_prompting.swap(0, Ordering::SeqCst);
                                    break 'done;
                                } // done
                                _ => unreachable!(),
                            }
                        }
                    }
                }
            }
        }
        status_tx
            .send(StatusUpdate::FetchThreadFinished)
            .unwrap_or_default();
    });

    let mut terminal = Terminal::new(TermionBackend::new(stdout()))?;
    terminal.clear()?;

    let mut app = TuiApp::new(recipes);

    let spinner = ['-', '\\', '|', '/'];
    let mut spinner_i = 0;

    while running.load(Ordering::SeqCst) {
        let frame_start = Instant::now();
        terminal.draw(|f| {
            spinner_i = (spinner_i + 1) % spinner.len();
            let spin = spinner[spinner_i];

            let mut constraints = Vec::new();
            if !app.fetch_complete {
                constraints.push(Constraint::Length(22));
            }
            constraints.push(Constraint::Length(22));
            constraints.push(Constraint::Min(20));
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(constraints)
                .split(f.area());
            let panel_height = chunks[0].height.saturating_sub(2) as usize;

            // Left Pane
            let fetch_items: Vec<ListItem> = app
                .recipes
                .iter()
                .filter(|(_, s)| *s == RecipeStatus::Pending || *s == RecipeStatus::Fetching)
                .map(|(r, s)| {
                    let style = if *s == RecipeStatus::Fetching {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    };
                    let icon = match s {
                        RecipeStatus::Pending => ' ',
                        RecipeStatus::Fetching => spin,
                        _ => '?',
                    };

                    ListItem::new(format!("{icon} {}", r.name)).style(style)
                })
                .collect();
            let fetch_list = List::new(fetch_items).block(
                Block::default()
                    .title("Fetch Queue [1]")
                    .borders(Borders::ALL),
            );
            f.render_widget(fetch_list, chunks[0]);

            // Right Pane
            let cook_items: Vec<ListItem> = app
                .recipes
                .iter()
                .filter(|(_, s)| {
                    *s == RecipeStatus::Fetched
                        || *s == RecipeStatus::Cooking
                        || *s == RecipeStatus::Done
                        || matches!(s, RecipeStatus::Failed(_))
                })
                .map(|(r, s)| {
                    let style = match s {
                        RecipeStatus::Fetched => Style::default().fg(Color::Cyan),
                        RecipeStatus::Cooking => Style::default().fg(Color::Yellow),
                        RecipeStatus::Done => Style::default().fg(Color::Green),
                        RecipeStatus::Failed(_) => Style::default().fg(Color::Red),
                        _ => Style::default(),
                    };
                    let icon = match s {
                        RecipeStatus::Fetched => ' ',
                        RecipeStatus::Cooking => spin,
                        RecipeStatus::Done => ' ',
                        RecipeStatus::Failed(_) => 'X',
                        _ => '?',
                    };
                    ListItem::new(format!("{icon} {}", r.name)).style(style)
                })
                .collect();
            let total_items = cook_items.len();
            if app.cook_auto_scroll {
                let cooking_index = app
                    .recipes
                    .iter()
                    .filter(|(_, s)| {
                        *s == RecipeStatus::Fetched
                            || *s == RecipeStatus::Cooking
                            || *s == RecipeStatus::Done
                            || matches!(s, RecipeStatus::Failed(_))
                    })
                    .position(|(_r, s)| *s == RecipeStatus::Cooking);

                if let Some(index) = cooking_index {
                    app.cook_list_state.select(Some(index));
                    let index_u16 = index;
                    let center_offset = panel_height / 2;
                    let new_offset = index_u16.saturating_sub(center_offset) as usize;

                    *app.cook_list_state.offset_mut() = new_offset;
                }
            } else {
                app.cook_list_state.select(None);
                if total_items > 0 {
                    let max_offset = total_items.saturating_sub(panel_height as usize);
                    if *app.cook_list_state.offset_mut() > max_offset {
                        *app.cook_list_state.offset_mut() = max_offset;
                    }
                } else {
                    *app.cook_list_state.offset_mut() = 0;
                }
            }
            let cook_items: Vec<ListItem> = cook_items[app.cook_scroll..].into();
            let cook_chunk = chunks[if app.fetch_complete { 0 } else { 1 }];
            let cook_list = List::new(cook_items).block(
                Block::default()
                    .title("Cook Queue [2]")
                    .borders(Borders::ALL),
            );
            f.render_stateful_widget(cook_list, cook_chunk, &mut app.cook_list_state);

            let (active_name, log_text, log_line) = app.get_active_log();
            let log_title = if let Some(active_name) = active_name {
                format!(
                    " {} Log: {} ",
                    app.log_view_job.to_string(),
                    active_name.as_str()
                )
            } else {
                format!(" {} Log ", app.log_view_job.to_string())
            };

            let mut enable_auto_scroll = false;
            let mut intended_scroll_pos = 0usize;

            let mut log_lines: Vec<Line> = if let Some(log_text) = log_text
                && !log_text.is_empty()
            {
                let total_log_lines = log_text.len() as usize;

                let start = if app.auto_scroll {
                    if total_log_lines > panel_height {
                        intended_scroll_pos = total_log_lines - panel_height;
                        total_log_lines - panel_height
                    } else {
                        0
                    }
                } else {
                    if total_log_lines > panel_height {
                        if app.log_scroll >= total_log_lines - panel_height {
                            enable_auto_scroll = true;
                            intended_scroll_pos = total_log_lines - panel_height;
                            total_log_lines - panel_height
                        } else {
                            app.log_scroll
                        }
                    } else {
                        0
                    }
                };

                let end = cmp::min(panel_height + start, total_log_lines - 1);

                log_text[start..end]
                    .iter()
                    .map(|s| {
                        let text_with_colors = s
                            .into_text()
                            .unwrap_or_else(|_| Text::raw("--unrenderable line--"));
                        text_with_colors
                            .lines
                            .into_iter()
                            .next()
                            .unwrap_or_else(|| Line::raw("--unrenderable line--"))
                    })
                    .collect()
            } else {
                vec![Line::from("No logs yet")]
            };

            if let Some(buffer) = log_line
                && !buffer.is_empty()
            {
                let text_with_colors = handle_cr(&buffer)
                    .into_text()
                    .unwrap_or_else(|_| Text::raw("--unrenderable line--"));

                if let Some(line) = text_with_colors.lines.into_iter().next() {
                    log_lines.push(line);
                }
            }

            let instruct = format!(
                " Keys: [c] Stop [PageUp/Down] Scroll{}{} ",
                match app.auto_scroll {
                    true => "",
                    false => " [End] Follow log trails",
                },
                match (&app.log_view_job, app.fetch_complete) {
                    (JobType::Fetch, _) => " [2] View Cook Log",
                    (JobType::Cook, false) => " [1] View Fetch Log",
                    (JobType::Cook, true) => "",
                }
            );

            let log_paragraph = Paragraph::new(log_lines)
                .block(
                    Block::default()
                        .title(log_title)
                        .title_bottom(instruct)
                        .borders(Borders::ALL),
                )
                .wrap(Wrap { trim: false });

            f.render_widget(
                log_paragraph,
                chunks[if app.fetch_complete { 1 } else { 2 }],
            );
            if let Some(prompt) = &mut app.prompt {
                if config.cook.nonstop && prompt.selected == PromptOption::Retry {
                    prompt.selected = PromptOption::Skip;
                }
                draw_prompt(f, prompt, config.cook.nonstop);
            }
            if enable_auto_scroll {
                app.auto_scroll = true;
            }
            if intended_scroll_pos > 0 {
                app.log_scroll = intended_scroll_pos;
            }

            while let Ok(event) = input_rx.try_recv() {
                if let Some((app, res)) = handle_prompt_input(&event, &mut app) {
                    prompting.swap(res as u32, Ordering::SeqCst);
                    if res == PromptOption::Exit {
                        // TODO: This can be a different log with what prompted on nonstop mode
                        let (name, log, line) = app.get_active_log();
                        if let Some(name) = name
                            && let Some(log) = log
                        {
                            app.dump_logs_on_exit = Some((name.to_owned(), join_logs(log, line)));
                        }
                        running.store(false, Ordering::SeqCst);
                    }
                    app.prompt = None;
                } else {
                    handle_main_event(&mut app, &event);
                }
            }
        })?;

        while let Ok(update) = status_rx.try_recv() {
            app.update_status(update);
        }

        if app.cook_complete {
            running.swap(false, Ordering::SeqCst);
        }

        if let Some(sleep_duration) = TICK_RATE.checked_sub(frame_start.elapsed()) {
            thread::sleep(sleep_duration);
        }
    }

    drop(mstdout);
    let _ = stdout().flush();

    if config.cook.nonstop && app.dump_logs_on_exit.is_some() {
        kill_everything();
    }

    fetcher_handle.join().unwrap();
    cooker_handle.join().unwrap();

    Ok(app.dump_logs_on_exit)
}

fn join_logs(log: &Vec<String>, line: Option<Cow<'_, str>>) -> String {
    let mut logs = log.join("\n");
    if let Some(line) = line {
        logs.push_str("\n");
        logs.push_str(handle_cr(&line));
    }
    logs
}

fn handle_cr<'a>(buffer: &'a Cow<'_, str>) -> &'a str {
    let st = buffer.trim_end();
    st.rsplit('\r').next().unwrap_or(&st)
}

fn handle_main_event(app: &mut TuiApp, event: &Event) {
    match event {
        Event::Key(key) => match key {
            Key::Char('1') => {
                app.log_view_job = JobType::Fetch;
            }
            Key::Char('2') => {
                app.log_view_job = JobType::Cook;
            }
            Key::Char('c') => {
                // as compilers still running, we use this way to stop it
                kill_everything();
            }
            Key::Up => {
                app.auto_scroll = false;
                app.log_scroll = app.log_scroll.saturating_sub(1);
            }
            Key::Down => {
                app.auto_scroll = false;
                app.log_scroll = app.log_scroll.saturating_add(1);
            }
            Key::PageUp => {
                app.auto_scroll = false;
                app.log_scroll = app.log_scroll.saturating_sub(20);
            }
            Key::PageDown => {
                app.auto_scroll = false;
                app.log_scroll = app.log_scroll.saturating_add(20);
            }
            Key::End => {
                app.auto_scroll = true;
            }
            Key::Home => {
                app.auto_scroll = false;
                app.log_scroll = 0;
            }
            _ => {}
        },

        //FIXME: This does nothing, it seems ratatui handles this itself magically
        Event::Mouse(mouse_event) => {
            match mouse_event {
                MouseEvent::Press(termion::event::MouseButton::WheelUp, x, y) => {
                    // termion is 1-based, ratatui rects are 0-based
                    let pos = Position {
                        x: x.saturating_sub(1),
                        y: y.saturating_sub(1),
                    };

                    if app.fetch_panel_rect.map_or(false, |r| r.contains(pos)) {
                        app.fetch_scroll = app.fetch_scroll.saturating_sub(1);
                    } else if app.cook_panel_rect.map_or(false, |r| r.contains(pos)) {
                        app.cook_scroll = app.cook_scroll.saturating_sub(1);
                        app.cook_auto_scroll = false;
                    } else if app.log_panel_rect.map_or(false, |r| r.contains(pos)) {
                        app.auto_scroll = false;
                        app.log_scroll = app.log_scroll.saturating_sub(1);
                    }
                }
                MouseEvent::Press(termion::event::MouseButton::WheelDown, x, y) => {
                    let pos = Position {
                        x: x.saturating_sub(1),
                        y: y.saturating_sub(1),
                    };

                    if app.fetch_panel_rect.map_or(false, |r| r.contains(pos)) {
                        app.fetch_scroll = app.fetch_scroll.saturating_add(1);
                    } else if app.cook_panel_rect.map_or(false, |r| r.contains(pos)) {
                        app.cook_scroll = app.cook_scroll.saturating_add(1);
                        app.cook_auto_scroll = false;
                    } else if app.log_panel_rect.map_or(false, |r| r.contains(pos)) {
                        app.auto_scroll = false;
                        app.log_scroll = app.log_scroll.saturating_add(1);
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn kill_everything() {
    let pid = std::process::id();
    Command::new("bash")
        .arg("-c")
        .arg(KILL_ALL_PID.replace("$PID", &pid.to_string()))
        .spawn()
        .expect("unable to spawn kill");
}

fn handle_prompt_input<'a>(
    event: &Event,
    app: &'a mut TuiApp,
) -> Option<(&'a mut TuiApp, PromptOption)> {
    if let Some(prompt) = &mut app.prompt {
        match event {
            Event::Key(key) => match key {
                Key::Char('q') | Key::Ctrl('c') | Key::Esc => {
                    // Treat as "Exit"
                    return Some((app, PromptOption::Exit));
                }
                Key::Left | Key::BackTab => prompt.prev(),
                Key::Right | Key::Char('\t') => prompt.next(),
                Key::Char('\n') => {
                    let prompt = app.prompt.take().unwrap();
                    return Some((app, prompt.selected));
                }
                _ => {}
            },
            _ => {} // Ignore mouse events
        }
    }
    None
}

fn draw_prompt(f: &mut ratatui::Frame, prompt: &FailurePrompt, is_nonstop: bool) {
    let title = format!(
        " FAILURE in {} {}",
        prompt.recipe.name,
        if is_nonstop { "(skipped) " } else { "" }
    );
    let mut error_text = prompt.error.clone();
    if error_text.len() > 200 {
        error_text = error_text[0..100].to_string()
            + ".."
            + &error_text[(error_text.len() - 100)..(error_text.len() - 1)];
    } else if error_text.len() > 100 {
        error_text = error_text[0..100].to_string() + "..";
    }

    // Style for options
    let retry_style = if prompt.selected == PromptOption::Retry {
        Style::default().bg(Color::White).fg(Color::Black)
    } else {
        Style::default()
    };
    let skip_style = if prompt.selected == PromptOption::Skip {
        Style::default().bg(Color::White).fg(Color::Black)
    } else {
        Style::default()
    };
    let exit_style = if prompt.selected == PromptOption::Exit {
        Style::default().bg(Color::White).fg(Color::Black)
    } else {
        Style::default()
    };

    let mut buttons = vec![
        Span::styled(" [Skip] ", skip_style),
        Span::raw("   "),
        Span::styled(" [Exit] ", exit_style),
    ];

    if !is_nonstop {
        buttons.push(Span::raw("   "));
        buttons.push(Span::styled(" [Retry] ", retry_style));
    }

    let text = vec![
        Line::from(error_text).style(Style::default().fg(Color::Yellow)),
        Line::from(""),
        Line::from(buttons),
    ];

    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default().fg(Color::White).bg(Color::Red),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center)
        .wrap(Wrap { trim: true });

    let area = f.area();
    let popup_area = Rect {
        x: area.width / 4,
        y: area.height / 3,
        width: area.width / 2,
        height: 10,
    };

    f.render_widget(Clear, popup_area); // Clear the background
    f.render_widget(paragraph, popup_area);
}

fn spawn_log_reader<R>(
    mut reader: R,
    package_name: PackageName,
    status_tx: mpsc::Sender<StatusUpdate>,
) where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut buffer = [0; 1024];
        loop {
            let buf = match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => buffer[..n].to_vec(),
                Err(e) => format!("[IO Error] {}", e).into_bytes(),
            };
            if status_tx
                .send(StatusUpdate::PushLog(package_name.clone(), buf))
                .is_err()
            {
                // TUI thread hung up
                break;
            }
        }
    });
}

fn setup_logger(
    status_tx: &mpsc::Sender<StatusUpdate>,
    name: &PackageName,
) -> (UnixSlavePty, std::io::PipeWriter) {
    let (pty_reader, log_reader, pipes) = setup_pty();

    spawn_log_reader(pty_reader, name.clone(), status_tx.clone());
    spawn_log_reader(log_reader, name.clone(), status_tx.clone());
    pipes
}

#[derive(PartialEq, Clone, Copy)]
#[repr(u32)]
enum PromptOption {
    Retry = 2,
    Skip,
    Exit,
}

struct FailurePrompt {
    recipe: CookRecipe,
    error: String,
    selected: PromptOption,
}

impl FailurePrompt {
    fn new(recipe: CookRecipe, error: String) -> Self {
        Self {
            recipe,
            error,
            selected: PromptOption::Exit,
        }
    }

    fn next(&mut self) {
        self.selected = match self.selected {
            PromptOption::Retry => PromptOption::Skip,
            PromptOption::Skip => PromptOption::Exit,
            PromptOption::Exit => PromptOption::Retry,
        }
    }

    fn prev(&mut self) {
        self.selected = match self.selected {
            PromptOption::Retry => PromptOption::Exit,
            PromptOption::Skip => PromptOption::Retry,
            PromptOption::Exit => PromptOption::Skip,
        }
    }
}
