use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    env, fs,
    path::{Path, PathBuf},
};

use pkg::{Package, PackageName};

use crate::{
    recipe::CookRecipe,
    web::html::{generate_html_index, generate_html_pkg},
};

pub mod html;

#[derive(Clone)]
pub struct CliWebConfig {
    /// path relative to cwd dir to generate web files
    out_dir: PathBuf,
    /// absolute url to repo (not the web) instead of "/repo"
    repo_url: String,
    /// this repository build url
    this_repo: String,
}

impl CliWebConfig {
    pub fn parse_args() -> Option<CliWebConfig> {
        if env::var("COOKBOOK_WEB").ok().as_deref() != Some("true") {
            return None;
        }
        let Ok(pwd) = env::current_dir() else {
            return None;
        };

        Some(CliWebConfig {
            repo_url: env::var("COOKBOOK_WEB_REPO_URL")
                .ok()
                .unwrap_or("/repo".to_string()),
            out_dir: pwd.join(
                env::var("COOKBOOK_WEB_OUT_DIR")
                    .ok()
                    .unwrap_or("web".to_string()),
            ),
            // TODO: Hardcoded URL, maybe get this remote-url next time
            this_repo: "https://gitlab.redox-os.org/redox-os/redox".to_string(),
        })
    }
}

const CSS: &str = include_str!("./web/style.css");

pub fn generate_web(all_packages: &Vec<String>, config: &CliWebConfig) {
    let repo_path = &config.out_dir.join(redoxer::target());
    if !repo_path.is_dir() {
        fs::create_dir_all(repo_path).unwrap();
    }

    let mut valid_packages = Vec::new();
    let mut dependents_map: HashMap<String, BTreeSet<String>> = HashMap::new();

    for package_name in all_packages {
        let Some(recipe_path) = pkg::recipes::find(package_name) else {
            continue;
        };
        // TODO: Package::from_path
        let Ok(package) = Package::new(&PackageName::new(package_name).unwrap()) else {
            continue;
        };
        let Ok(recipe) = CookRecipe::from_path(&recipe_path, true, false) else {
            continue;
        };

        for dep in &package.depends {
            dependents_map
                .entry(dep.to_string())
                .or_default()
                .insert(package.name.to_string());
        }

        valid_packages.push((package, recipe));
    }

    for (package, recipe) in &valid_packages {
        let dependents = dependents_map
            .get(package.name.as_str())
            .cloned()
            .unwrap_or_default();

        let stage_files_path = recipe.stage_paths().0.with_added_extension("files");
        let stage_files = fs::read_to_string(stage_files_path).ok();

        let html_path = repo_path.join(format!("{}.html", package.name.as_str()));

        generate_html_pkg(
            &package,
            &recipe,
            &dependents.into_iter().collect(),
            &stage_files,
            &html_path,
            &config,
        );
    }

    let mut grouped_packages: BTreeMap<String, Vec<&(Package, CookRecipe)>> = BTreeMap::new();

    for item in &valid_packages {
        let category = get_category(&item.1.dir);
        grouped_packages.entry(category).or_default().push(item);
    }

    let index_path = repo_path.join("index.html");
    let style_path = repo_path.join("style.css");
    generate_html_index(grouped_packages, &index_path, &config);
    fs::write(style_path, CSS).expect("Failed to write CSS file");
}

pub(crate) fn get_category(dir: &Path) -> String {
    let Some(category) = dir.parent().map(|p| p.display().to_string()) else {
        return "uncategorized".to_string();
    };
    category["recipes/".len()..].to_string()
}
