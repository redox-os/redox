use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::LazyLock;

static RECIPE_PATHS: LazyLock<HashMap<String, PathBuf>> = LazyLock::new(|| {
    let mut recipe_paths = HashMap::new();
    for entry_res in ignore::Walk::new("recipes") {
        let entry = entry_res.unwrap();
        if entry.file_name() == OsStr::new("recipe.sh") || entry.file_name() == OsStr::new("recipe.toml") {
            let recipe_file = entry.path();
            let Some(recipe_dir) = recipe_file.parent() else { continue };
            let Some(recipe_name) = recipe_dir.file_name().and_then(|x| x.to_str()) else { continue };
            if let Some(other_dir) = recipe_paths.insert(recipe_name.to_string(), recipe_dir.to_path_buf()) {
                panic!(
                    "recipe {} has two or more entries {:?}, {:?}",
                    recipe_name,
                    other_dir,
                    recipe_dir
                );
            }
        }
    }
    recipe_paths
});

pub fn recipe_find(recipe: &str) -> Option<PathBuf> {
    RECIPE_PATHS.get(recipe).cloned()
}

pub fn list_recipes(prefix: PathBuf) -> Vec<PathBuf> {
    let mut recipes = Vec::<PathBuf>::new();
    for (_name, path) in RECIPE_PATHS.iter() {
        recipes.push(prefix.join(path));
    }
    recipes.sort();
    recipes
}
