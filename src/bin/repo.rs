use std::collections::HashMap;
use std::io::{BufRead, BufReader, PipeReader, stdout};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::time::Duration;
use std::{env, fs};
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
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use termion::event::{Event, Key, MouseEvent};
use termion::input::{MouseTerminal, TermRead};
use termion::raw::RawTerminal;
use termion::screen::{AlternateScreen, ToAlternateScreen, ToMainScreen};

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
    main_inner().unwrap();
}

fn main_inner() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        println!("{}", REPO_HELP_STR);
        process::exit(1);
    }

    let (config, command, recipe_names) = parse_args(args)?;

    if command == CliCommand::Cook && config.cook.tui {
        run_tui_cook(config, recipe_names)?;
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
    .map_err(|e| anyhow!(e))?;

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
    Fetched(PackageName),
    FailFetch(PackageName, String),
    StartCook(PackageName),
    CookLog(PackageName, String),
    Cooked(PackageName),
    FailCook(PackageName, String),
    FetchThreadFinished,
    CookThreadFinished,
}

struct TuiApp {
    recipes: Vec<(CookRecipe, RecipeStatus)>,
    fetch_queue: Vec<PackageName>,
    cook_queue: Vec<PackageName>,
    done: Vec<PackageName>,
    failed: Vec<PackageName>,
    active_fetch: Option<PackageName>,
    active_cook: Option<PackageName>,
    logs: HashMap<PackageName, Vec<String>>,
    log_scroll: u16,
    auto_scroll: bool,
    fetch_scroll: u16,
    cook_scroll: u16,
    fetch_complete: bool,
    cook_complete: bool,
    fetch_panel_rect: Option<Rect>,
    cook_panel_rect: Option<Rect>,
    log_panel_rect: Option<Rect>,
}

impl TuiApp {
    fn new(recipes: Vec<CookRecipe>) -> Self {
        let recipe_names = recipes.iter().map(|r| r.name.clone()).collect();
        Self {
            recipes: recipes
                .into_iter()
                .map(|r| (r, RecipeStatus::Pending))
                .collect(),
            fetch_queue: recipe_names,
            cook_queue: Vec::new(),
            done: Vec::new(),
            failed: Vec::new(),
            active_fetch: None,
            active_cook: None,
            logs: HashMap::new(),
            log_scroll: 0,
            auto_scroll: true,
            fetch_scroll: 0,
            cook_scroll: 0,
            fetch_complete: false,
            cook_complete: false,
            fetch_panel_rect: None,
            cook_panel_rect: None,
            log_panel_rect: None,
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
            StatusUpdate::Fetched(name) => (name, RecipeStatus::Fetched),
            StatusUpdate::FailFetch(name, err) => (name, RecipeStatus::Failed(err)),
            StatusUpdate::StartCook(name) => {
                self.active_cook = Some(name.clone());
                self.logs.insert(name.clone(), Vec::new());
                (name.clone(), RecipeStatus::Cooking)
            }
            StatusUpdate::CookLog(name, line) => {
                self.logs.entry(name.clone()).or_default().push(line);
                // No status change, just return the current state
                if let Some((_, status)) = self.recipes.iter().find(|(r, _)| r.name == name) {
                    (name, status.clone())
                } else {
                    return; // Should not happen
                }
            }
            StatusUpdate::Cooked(name) => {
                if self.active_cook.as_ref() == Some(&name) {
                    self.active_cook = None;
                }
                self.auto_scroll = true;
                (name.clone(), RecipeStatus::Done)
            }
            StatusUpdate::FailCook(name, err) => {
                if self.active_cook.as_ref() == Some(&name) {
                    self.active_cook = None;
                }
                self.auto_scroll = false;
                (name.clone(), RecipeStatus::Failed(err))
            }
            StatusUpdate::FetchThreadFinished => {
                self.fetch_complete = true;
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
            .filter(|(_, s)| *s == RecipeStatus::Pending || *s == RecipeStatus::Fetching)
            .map(|(r, _)| r.name.clone())
            .collect();
        self.cook_queue = self
            .recipes
            .iter()
            .filter(|(_, s)| *s == RecipeStatus::Fetched || *s == RecipeStatus::Cooking)
            .map(|(r, _)| r.name.clone())
            .collect();
        self.done = self
            .recipes
            .iter()
            .filter(|(_, s)| *s == RecipeStatus::Done)
            .map(|(r, _)| r.name.clone())
            .collect();
        self.failed = self
            .recipes
            .iter()
            .filter(|(_, s)| matches!(s, RecipeStatus::Failed(_)))
            .map(|(r, _)| r.name.clone())
            .collect();
    }
}

fn run_tui_cook(config: CliConfig, recipes: Vec<CookRecipe>) -> anyhow::Result<()> {
    let (work_tx, work_rx) = mpsc::channel::<(CookRecipe, PathBuf)>();
    let (status_tx, status_rx) = mpsc::channel::<StatusUpdate>();

    // ---- Cooker Thread ----
    let cooker_config = config.clone();
    let cooker_status_tx = status_tx.clone();
    let cooker_handle = thread::spawn(move || {
        for (recipe, source_dir) in work_rx {
            let name = recipe.name.clone();
            let is_deps = recipe.is_deps;
            cooker_status_tx
                .send(StatusUpdate::StartCook(name.clone()))
                .unwrap();
            let (mut stdout_writer, mut stderr_writer) = setup_logger(&cooker_status_tx, &name);
            let logger = Some((&mut stdout_writer, &mut stderr_writer));
            match handle_cook(&recipe, &cooker_config, source_dir, is_deps, &logger) {
                Ok(_) => cooker_status_tx.send(StatusUpdate::Cooked(name)).unwrap(),
                Err(e) => cooker_status_tx
                    .send(StatusUpdate::FailCook(name, e.to_string()))
                    .unwrap(),
            }
        }
        cooker_status_tx
            .send(StatusUpdate::CookThreadFinished)
            .unwrap_or_default();
    });

    // ----- Input Thread -----
    let (input_tx, input_rx) = mpsc::channel::<Event>();
    let _input_handle = thread::spawn(move || {
        let stdin = std::io::stdin();
        for evt in stdin.events() {
            if let Ok(evt) = evt {
                if input_tx.send(evt).is_err() {
                    return;
                }
            }
        }
    });

    // ---- Fetcher Thread ----
    let fetcher_recipes = recipes.clone();
    let fetcher_config = config.clone();
    let fetcher_handle = thread::spawn(move || {
        for recipe in fetcher_recipes {
            let name = recipe.name.clone();
            status_tx
                .send(StatusUpdate::StartFetch(name.clone()))
                .unwrap();

            let (mut stdout_writer, mut stderr_writer) = setup_logger(&status_tx, &name);
            let logger = Some((&mut stdout_writer, &mut stderr_writer));

            match handle_fetch(&recipe, &fetcher_config, &logger) {
                Ok(source_dir) => {
                    status_tx.send(StatusUpdate::Fetched(name)).unwrap();
                    if work_tx.send((recipe.clone(), source_dir)).is_err() {
                        // Cooker thread died
                        break;
                    }
                }
                Err(e) => status_tx
                    .send(StatusUpdate::FailFetch(name, e.to_string()))
                    .unwrap(),
            }
            status_tx
                .send(StatusUpdate::FetchThreadFinished)
                .unwrap_or_default();
        }
    });

    print!("{}", ToAlternateScreen);
    let mut terminal = Terminal::new(TermionBackend::new(stdout()))?;
    terminal.clear()?;

    let mut app = TuiApp::new(recipes);
    // let total_recipes = app.recipes.len();
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .context("Error setting Ctrl-C handler")?;

    while running.load(Ordering::Relaxed) {
        terminal.draw(|f| {
            let mut constraints = Vec::new();
            if !app.fetch_complete {
                constraints.push(Constraint::Length(30));
            }
            constraints.push(Constraint::Length(30));
            constraints.push(Constraint::Min(20)); // Log panel always exists
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
            let fetch_list = List::new(fetch_items)
                .block(Block::default().title("Fetch Queue").borders(Borders::ALL));
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
            let cook_list = List::new(cook_items)
                .block(Block::default().title("Cook Queue").borders(Borders::ALL));
            f.render_widget(cook_list, chunks[1]);

            let log_title = if let Some(active_name) = &app.active_cook {
                format!("Build Log: {}", active_name.as_str())
            } else {
                "Build Log".to_string()
            };

            let log_text: Vec<String> = if let Some(active_name) = &app.active_cook {
                app.logs
                    .get(active_name)
                    .cloned()
                    .unwrap_or_else(|| vec!["Waiting for logs...".to_string()])
            } else {
                vec!["No active cook job.".to_string()]
            };

            let log_pane_height = chunks[2].height.saturating_sub(2);
            let total_log_lines = log_text.len() as u16;

            if app.auto_scroll {
                if total_log_lines > log_pane_height {
                    app.log_scroll = total_log_lines - log_pane_height;
                } else {
                    app.log_scroll = 0;
                }
            } else {
                if total_log_lines > log_pane_height {
                    if app.log_scroll > total_log_lines - log_pane_height {
                        app.log_scroll = total_log_lines - log_pane_height;
                    }
                } else {
                    app.log_scroll = 0;
                }
            }

            let log_paragraph = Paragraph::new(log_text.join("\n"))
                .block(Block::default().title(log_title).borders(Borders::ALL))
                .wrap(Wrap { trim: false })
                .scroll((app.log_scroll, 0));

            f.render_widget(log_paragraph, chunks[2]);

            while let Ok(event) = input_rx.try_recv() {
                match event {
                    Event::Key(key) => match key {
                        Key::Up => {
                            app.auto_scroll = false;
                            app.log_scroll = app.log_scroll.saturating_sub(1);
                        }
                        Key::Down => {
                            app.auto_scroll = false;
                            app.log_scroll = app.log_scroll.saturating_add(1);
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
        })?;

        while let Ok(update) = status_rx.try_recv() {
            app.update_status(update);
        }

        if fetcher_handle.is_finished() && cooker_handle.is_finished() {
            thread::sleep(Duration::from_secs(5));
            running.swap(false, Ordering::SeqCst);
        }
    }

    // disable_raw_mode()?;
    print!("{}", ToMainScreen);

    fetcher_handle.join().unwrap();
    cooker_handle.join().unwrap();

    Ok(())
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
                .send(StatusUpdate::CookLog(package_name.clone(), line_str))
                .is_err()
            {
                // TUI thread hung up
                break;
            }
        }
    });
}

fn setup_logger(
    cooker_status_tx: &mpsc::Sender<StatusUpdate>,
    name: &PackageName,
) -> (std::io::PipeWriter, std::io::PipeWriter) {
    let (stdout_reader, stdout_writer) = std::io::pipe().expect("Failed to create stdout pipe");
    let (stderr_reader, stderr_writer) = std::io::pipe().expect("Failed to create stderr pipe");
    spawn_log_reader(stdout_reader, name.clone(), cooker_status_tx.clone());
    spawn_log_reader(stderr_reader, name.clone(), cooker_status_tx.clone());
    (stdout_writer, stderr_writer)
}
