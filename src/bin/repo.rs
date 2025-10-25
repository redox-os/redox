use std::collections::{HashMap, VecDeque};
use std::io::{BufRead, BufReader, PipeReader, Write, stderr, stdin, stdout};
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, mpsc};
use std::time::Duration;
use std::{cmp, env, fs};
use std::{process, thread};

use anyhow::{Context, anyhow, bail};
use cookbook::WALK_DEPTH;
use cookbook::config::{CookConfig, get_config, init_config};
use cookbook::cook::cook_build::build;
use cookbook::cook::fetch::{fetch, fetch_offline};
use cookbook::cook::fs::{Stdout, create_target_dir};
use cookbook::cook::package::package;
use cookbook::recipe::CookRecipe;
use pkg::PackageName;
use pkg::package::PackageError;
use ratatui::Terminal;
use ratatui::layout::{Constraint, Direction, Layout, Position, Rect};
use ratatui::prelude::TermionBackend;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
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
        tree      show tree of recipe packages
    
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

#[derive(Clone)]
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
    Tree,
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
            CliCommand::Tree => "tree".to_string(),
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
        if let Some((name, e)) = run_tui_cook(config, recipe_names)? {
            let _ = stderr().write(e.as_bytes());
            let _ = stderr().write(b"\n\n");
            eprintln!(
                "{}{}cook - failed at {}{}{}",
                style::Bold,
                color::Fg(color::AnsiValue(196)),
                name.as_str(),
                color::Fg(color::Reset),
                style::Reset,
            );
            return Err(anyhow!("Execution has failed"));
        } else {
            eprintln!(
                "{}{}cook - successful{}{}",
                style::Bold,
                color::Fg(color::AnsiValue(46)),
                color::Fg(color::Reset),
                style::Reset,
            );
        }
        return Ok(());
    }

    for recipe in &recipe_names {
        match command {
            CliCommand::Fetch => {
                handle_fetch(recipe, &config, &None)?;
            }
            CliCommand::Cook => {
                let source_dir = handle_fetch(recipe, &config, &None)?;
                handle_cook(recipe, &config, source_dir, recipe.is_deps, &None)?
            }
            CliCommand::Unfetch => handle_clean(recipe, &config, true, true)?,
            CliCommand::Clean => handle_clean(recipe, &config, false, true)?,
            CliCommand::Push => handle_push(recipe, &config)?,
            CliCommand::Tree => todo!("tree command is WIP"),
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
            || command == CliCommand::Tree
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

fn handle_fetch(
    recipe: &CookRecipe,
    config: &CliConfig,
    logger: &Stdout,
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
    logger: &Stdout,
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

#[derive(Debug, Clone)]
enum StatusUpdate {
    StartFetch(PackageName),
    Fetched(CookRecipe),
    FailFetch(CookRecipe, String),
    StartCook(PackageName),
    Cooked(CookRecipe),
    FailCook(CookRecipe, String),
    PushLog(PackageName, String),
    FetchThreadFinished,
    CookThreadFinished,
}

#[derive(PartialEq)]
enum JobType {
    Fetch,
    Cook,
}

struct TuiApp {
    recipes: Vec<(CookRecipe, RecipeStatus)>,
    fetch_queue: VecDeque<CookRecipe>,
    cook_queue: VecDeque<CookRecipe>,
    done: Vec<PackageName>,
    active_fetch: Option<PackageName>,
    active_cook: Option<PackageName>,
    logs: HashMap<PackageName, Vec<String>>,
    log_scroll: usize,
    log_view_job: JobType,
    auto_scroll: bool,
    fetch_scroll: usize,
    cook_scroll: usize,
    fetch_complete: bool,
    cook_complete: bool,
    fetch_panel_rect: Option<Rect>,
    cook_panel_rect: Option<Rect>,
    log_panel_rect: Option<Rect>,
    prompt: Option<FailurePrompt>,
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
            log_scroll: 0,
            auto_scroll: true,
            log_view_job: JobType::Fetch,
            fetch_scroll: 0,
            cook_scroll: 0,
            fetch_complete: false,
            cook_complete: false,
            fetch_panel_rect: None,
            cook_panel_rect: None,
            log_panel_rect: None,
            prompt: None,
            dump_logs_on_exit: None,
        }
    }

    // Update the state based on a message from a worker thread
    fn update_status(&mut self, update: StatusUpdate) {
        let (name, new_status) = match update {
            StatusUpdate::StartFetch(name) => {
                self.active_fetch = Some(name.clone());
                self.logs.insert(name.clone(), Vec::new());
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
                (name.clone(), RecipeStatus::Cooking)
            }
            StatusUpdate::PushLog(name, line) => {
                self.logs.entry(name.clone()).or_default().push(line);
                // No status change, just return the current state
                if let Some((_, status)) = self.recipes.iter().find(|(r, _)| r.name == name) {
                    (name, status.clone())
                } else {
                    return; // Should not happen
                }
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

    // ---- Cooker Thread ----
    let cooker_config = config.clone();
    let cooker_status_tx = status_tx.clone();
    let cooker_prompting = prompting.clone();
    let cooker_handle = thread::spawn(move || {
        'done: for (recipe, source_dir) in work_rx {
            let name = recipe.name.clone();
            let is_deps = recipe.is_deps;
            cooker_status_tx
                .send(StatusUpdate::StartCook(name.clone()))
                .unwrap();
            let (mut stdout_writer, mut stderr_writer) = setup_logger(&cooker_status_tx, &name);
            let logger = Some((&mut stdout_writer, &mut stderr_writer));
            'again: loop {
                match handle_cook(
                    &recipe,
                    &cooker_config,
                    source_dir.clone(),
                    is_deps,
                    &logger,
                ) {
                    Ok(()) => {
                        cooker_status_tx
                            .send(StatusUpdate::Cooked(recipe))
                            .unwrap_or_default();
                        break;
                    }
                    Err(e) => {
                        cooker_status_tx
                            .send(StatusUpdate::FailCook(recipe.clone(), e.to_string()))
                            .unwrap_or_default();
                        if !cooker_config.cook.nonstop {
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
            fetcher_status_tx
                .send(StatusUpdate::StartFetch(name.clone()))
                .unwrap();

            let (mut stdout_writer, mut stderr_writer) = setup_logger(&fetcher_status_tx, &name);
            let logger = Some((&mut stdout_writer, &mut stderr_writer));

            'again: loop {
                match handle_fetch(&recipe, &fetcher_config, &logger) {
                    Ok(source_dir) => {
                        fetcher_status_tx
                            .send(StatusUpdate::Fetched(recipe.clone()))
                            .unwrap();
                        if work_tx.send((recipe.clone(), source_dir)).is_err() {
                            // Cooker thread died
                            break 'done;
                        }
                        break;
                    }
                    Err(e) => {
                        fetcher_status_tx
                            .send(StatusUpdate::FailFetch(recipe.clone(), e.to_string()))
                            .unwrap_or_default();
                        if !fetcher_config.cook.nonstop {
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
        }
        status_tx
            .send(StatusUpdate::FetchThreadFinished)
            .unwrap_or_default();
    });

    let mut terminal = Terminal::new(TermionBackend::new(stdout()))?;
    terminal.clear()?;

    let mut app = TuiApp::new(recipes);

    while running.load(Ordering::SeqCst) {
        terminal.draw(|f| {
            let mut constraints = Vec::new();
            if !app.fetch_complete {
                constraints.push(Constraint::Length(30));
            }
            constraints.push(Constraint::Length(30));
            constraints.push(Constraint::Min(20));
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(constraints)
                .split(f.area());

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
                    ListItem::new(r.name.as_str()).style(style)
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
                    ListItem::new(r.name.as_str()).style(style)
                })
                .collect();
            let cook_list = List::new(cook_items).block(
                Block::default()
                    .title("Cook Queue [2]")
                    .borders(Borders::ALL),
            );
            f.render_widget(cook_list, chunks[if app.fetch_complete { 0 } else { 1 }]);

            let (active_name, log_text) = get_active_log(&app);

            let log_title = if let Some(active_name) = active_name {
                format!("Build Log: {}", active_name.as_str())
            } else {
                "Build Log".to_string()
            };

            let mut enable_auto_scroll = false;

            let log_lines: Vec<Line> = if let Some(log_text) = log_text
                && log_text.len() > 0
            {
                let log_pane_height = chunks[if app.fetch_complete { 1 } else { 2 }]
                    .height
                    .saturating_sub(2) as usize;
                let total_log_lines = log_text.len() as usize;

                let start = if app.auto_scroll {
                    if total_log_lines > log_pane_height {
                        total_log_lines - log_pane_height
                    } else {
                        0
                    }
                } else {
                    if total_log_lines > log_pane_height {
                        if app.log_scroll >= total_log_lines - log_pane_height {
                            enable_auto_scroll = true;
                            total_log_lines - log_pane_height
                        } else {
                            app.log_scroll
                        }
                    } else {
                        0
                    }
                };

                let end = cmp::min(log_pane_height + start, total_log_lines - 1);

                log_text[start..end]
                    .iter()
                    .map(|s| Line::from(s.clone()))
                    .collect()
            } else {
                vec![Line::from("No logs yet")]
            };

            let log_paragraph = Paragraph::new(log_lines)
                .block(Block::default().title(log_title).borders(Borders::ALL))
                .wrap(Wrap { trim: false });

            f.render_widget(
                log_paragraph,
                chunks[if app.fetch_complete { 1 } else { 2 }],
            );
            if let Some(prompt) = &app.prompt {
                draw_prompt(f, prompt);
            }
            if enable_auto_scroll {
                app.auto_scroll = true;
            }

            while let Ok(event) = input_rx.try_recv() {
                if let Some((app, res)) = handle_prompt_input(&event, &mut app) {
                    prompting.swap(res as u32, Ordering::SeqCst);
                    if res == PromptOption::Exit {
                        let (name, log) = get_active_log(&app);
                        if let Some(name) = name
                            && let Some(log) = log
                        {
                            app.dump_logs_on_exit = Some((name.to_owned(), log.join("\n")));
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
    }

    drop(mstdout);
    let _ = stdout().flush();

    fetcher_handle.join().unwrap();
    cooker_handle.join().unwrap();

    Ok(app.dump_logs_on_exit)
}

fn get_active_log(app: &TuiApp) -> (Option<PackageName>, Option<&Vec<String>>) {
    let active_name = if app.log_view_job == JobType::Cook {
        app.active_cook.clone()
    } else {
        app.active_fetch.clone()
    };

    let log_text = if let Some(active_name) = &active_name {
        app.logs.get(active_name)
    } else {
        None
    };
    (active_name, log_text)
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
                let pid = std::process::id();
                Command::new("pkill")
                    .arg("-9")
                    .arg("-P")
                    .arg(pid.to_string())
                    .spawn()
                    .expect("unable to spawn pkill");
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

fn draw_prompt(f: &mut ratatui::Frame, prompt: &FailurePrompt) {
    let title = format!(" FAILURE in {} ", prompt.recipe.name);
    let mut error_text = prompt.error.clone();
    if error_text.len() > 100 {
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

    let text = vec![
        Line::from(error_text).style(Style::default().fg(Color::Yellow)),
        Line::from(""),
        Line::from(vec![
            Span::styled(" [Retry] ", retry_style),
            Span::raw("   "),
            Span::styled(" [Skip] ", skip_style),
            Span::raw("   "),
            Span::styled(" [Exit] ", exit_style),
        ]),
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

fn spawn_log_reader(
    mut pipe_reader: PipeReader,
    package_name: PackageName,
    status_tx: mpsc::Sender<StatusUpdate>,
) {
    thread::spawn(move || {
        let reader = BufReader::new(&mut pipe_reader);
        for line in reader.lines() {
            let line_str = line.unwrap_or_else(|e| format!("[IO Error] {}", e));
            if status_tx
                .send(StatusUpdate::PushLog(package_name.clone(), line_str))
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
) -> (std::io::PipeWriter, std::io::PipeWriter) {
    let (stdout_reader, stdout_writer) = std::io::pipe().expect("Failed to create stdout pipe");
    let (stderr_reader, stderr_writer) = std::io::pipe().expect("Failed to create stderr pipe");
    spawn_log_reader(stdout_reader, name.clone(), status_tx.clone());
    spawn_log_reader(stderr_reader, name.clone(), status_tx.clone());
    (stdout_writer, stderr_writer)
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
