// Contains code related to TUI model

use crate::*;

#[derive(PartialEq)]
pub enum RightPanelTab {
    MainActions,
    Terminal(usize),
}

#[derive(PartialEq, Clone, Copy)]
pub enum FilterSource {
    Config,
    Compiled,
    Pinned,
    All,
}

impl FilterSource {
    pub fn labels() -> Vec<&'static str> {
        vec!["[F2] Selected", "[F3] Config", "[F4] Compiled", "[F5] All"]
    }

    pub fn index(&self) -> usize {
        match self {
            FilterSource::Pinned => 0,
            FilterSource::Config => 1,
            FilterSource::Compiled => 2,
            FilterSource::All => 3,
        }
    }

    pub const fn try_from_index(i: usize) -> Option<FilterSource> {
        Some(match i {
            0 => FilterSource::Pinned,
            1 => FilterSource::Config,
            2 => FilterSource::Compiled,
            3 => FilterSource::All,
            _ => return None,
        })
    }
}

#[derive(PartialEq)]
pub enum Focus {
    SearchAndList,
    MainPanel,
}

pub struct ActionButton {
    pub key: char,
    pub label: &'static str,
    pub cmd: &'static str,
    pub area: Rect,
}

pub struct App {
    pub search_query: String,
    pub all_recipes: Vec<CookRecipe>,
    pub all_compiled_recipes: Vec<CookRecipe>,
    pub all_config_recipes: Vec<CookRecipe>,
    pub filtered_recipes: Vec<PackageName>,
    pub pinned_recipes: BTreeSet<PackageName>,
    pub filter_source: FilterSource,
    pub list_state: ListState,
    pub focus: Focus,
    pub buttons: Vec<ActionButton>,

    pub last_click_time: Option<Instant>,
    pub last_click_idx: Option<usize>,

    pub filter_tabs_area: Rect,
    pub right_tabs_area: Rect,
    pub right_tabs_str: Vec<String>,
    pub right_tab_active: RightPanelTab,
    pub exec: ExecutionManager,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            search_query: String::new(),
            all_recipes: Vec::new(),
            all_compiled_recipes: Vec::new(),
            all_config_recipes: Vec::new(),
            filtered_recipes: Vec::new(),
            pinned_recipes: BTreeSet::new(),
            filter_source: FilterSource::Config,
            list_state: ListState::default(),
            right_tabs_area: Rect::default(),
            right_tabs_str: Vec::new(),
            focus: Focus::SearchAndList,
            buttons: vec![
                ActionButton {
                    key: 'f',
                    label: "[F]etch",
                    cmd: "fetch",
                    area: Rect::default(),
                },
                ActionButton {
                    key: 'u',
                    label: "[U]nfetch",
                    cmd: "unfetch",
                    area: Rect::default(),
                },
                ActionButton {
                    key: 'c',
                    label: "[C]lean",
                    cmd: "clean",
                    area: Rect::default(),
                },
                ActionButton {
                    key: 'r',
                    label: "[R]ebuild",
                    cmd: "rebuild",
                    area: Rect::default(),
                },
                ActionButton {
                    key: 'p',
                    label: "[P]ush",
                    cmd: "push",
                    area: Rect::default(),
                },
            ],
            last_click_time: None,
            last_click_idx: None,
            filter_tabs_area: Rect::default(),
            right_tab_active: RightPanelTab::MainActions,
            exec: ExecutionManager::new(),
        };
        app.read_recipes();
        app.update_filter();
        app
    }

    pub fn read_recipes(&mut self) {
        self.all_recipes = staged_pkg::list()
            .iter()
            .filter_map(|p| CookRecipe::from_path(p, false, false).ok())
            .collect();
        self.all_compiled_recipes = staged_pkg::list_repo("repo")
            .unwrap()
            .iter()
            .filter_map(|p| CookRecipe::from_path(p, false, false).ok())
            .collect();
        self.all_config_recipes = std::env::var("CONFIG")
            .ok()
            .and_then(|s| redox_installer::Config::from_file(Path::new(&s)).ok())
            .map(|s| {
                s.packages
                    .keys()
                    .filter_map(|p| PackageName::new(p).ok())
                    .filter_map(|p| CookRecipe::from_name(p).ok())
                    .collect()
            })
            .unwrap_or(vec![]);
    }

    pub fn source_filter(&self) -> &Vec<CookRecipe> {
        match self.filter_source {
            FilterSource::Config => &self.all_config_recipes,
            FilterSource::Compiled => &self.all_compiled_recipes,
            FilterSource::All => &self.all_recipes,
            _ => todo!(),
        }
    }

    pub fn update_filter(&mut self) {
        let query = self.search_query.to_lowercase();

        if matches!(self.filter_source, FilterSource::Pinned) {
            self.filtered_recipes = self.pinned_recipes.iter().cloned().collect();
        } else {
            self.filtered_recipes = self
                .source_filter()
                .iter()
                .filter(|p| p.name.as_str().contains(&query))
                .map(|r| r.name.clone())
                .collect();
        }

        self.filtered_recipes.sort();

        if self.filtered_recipes.is_empty() {
            self.list_state.select(None);
        } else if self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
        } else if let Some(idx) = self.list_state.selected() {
            if idx >= self.filtered_recipes.len() {
                self.list_state
                    .select(Some(self.filtered_recipes.len() - 1));
            }
        }

        self.right_tab_active = RightPanelTab::MainActions;
    }

    pub fn highlighted_recipe(&self) -> Option<&PackageName> {
        self.list_state
            .selected()
            .and_then(|idx| self.filtered_recipes.get(idx))
    }

    pub fn selected_recipes(&self) -> Vec<PackageName> {
        if !self.pinned_recipes.is_empty() {
            let mut targets: Vec<_> = self.pinned_recipes.iter().cloned().collect();
            targets.sort();
            targets
        } else if let Some(single) = self.highlighted_recipe() {
            vec![single.clone()]
        } else {
            Vec::new()
        }
    }

    pub fn toggle_pinned_highlighted(&mut self) {
        if let Some(recipe) = self.highlighted_recipe().cloned() {
            if self.pinned_recipes.contains(&recipe) {
                self.pinned_recipes.remove(&recipe);
            } else {
                self.pinned_recipes.insert(recipe);
            }
        }
    }

    pub fn pin_all_filtered(&mut self) {
        let mut nothing_pinned = true;
        for recipe in &self.filtered_recipes {
            nothing_pinned &= !self.pinned_recipes.insert(recipe.clone());
        }
        if nothing_pinned {
            for recipe in &self.filtered_recipes {
                self.pinned_recipes.remove(recipe);
            }
        }
    }

    pub fn box_style<'a, 'b>(&'b self, title: &'a str, hint: &'a str, focus: Focus) -> Block<'a> {
        let style = if self.focus == focus {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_bottom(hint)
            .border_style(style)
    }

    pub fn select_delta(&mut self, delta: isize, wrap: bool) {
        let len = self.filtered_recipes.len();
        if len == 0 {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let next = current.saturating_add_signed(delta);
        let next = if wrap {
            next % len
        } else {
            next.clamp(0, len - 1)
        };

        self.list_state.select(Some(next));
        self.focus = Focus::SearchAndList;
        self.right_tab_active = RightPanelTab::MainActions;
    }

    pub fn current_job_id(&self) -> Option<JobId> {
        let RightPanelTab::Terminal(i) = self.right_tab_active else {
            return None;
        };
        self.exec.active_job_order.get(i).map(|s| *s)
    }

    pub fn current_job(&self) -> Option<&PackageJob> {
        let Some(i) = self.current_job_id() else {
            return None;
        };
        self.exec.jobs.get(&i)
    }

    pub fn current_job_mut(&mut self) -> Option<&mut PackageJob> {
        let Some(i) = self.current_job_id() else {
            return None;
        };
        self.exec.jobs.get_mut(&i)
    }
}
