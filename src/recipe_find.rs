use std::ffi::OsStr;
use std::fs::{self};
use std::path::{Path, PathBuf};

pub fn recipe_find(recipe: &str, dir: &Path) -> Result<Option<PathBuf>, String> {
    let mut recipe_path = None;
    if !dir.is_dir() {
        return Ok(None);
    }
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_name() == OsStr::new("recipe.sh")
            || entry.file_name() == OsStr::new("recipe.toml")
        {
            // println!("recipe is {:?}", dir.file_name());
            if dir.file_name().unwrap() != OsStr::new(recipe) {
                return Ok(None);
            } else {
                return Ok(Some(dir.to_path_buf()));
            }
        }
    }

    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            continue;
        }
        let found = recipe_find(recipe, entry.path().as_path())?;
        if found.is_none() {
            continue;
        }
        if recipe_path.is_none() {
            recipe_path = found;
        } else {
            return Err(format!(
                "recipe {} has two or more entries {}, {}",
                recipe,
                recipe_path.unwrap().display(),
                found.unwrap().display()
            ));
        }
    }

    Ok(recipe_path)
}

pub fn list_recipes(dir: &Path) -> Result<Vec<String>, String> {
    let mut recipes = Vec::<String>::new();
    if !dir.is_dir() {
        return Ok(recipes);
    }
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_name() == OsStr::new("recipe.sh")
            || entry.file_name() == OsStr::new("recipe.toml")
        {
            recipes.push(dir.file_name().ok_or(format!("could not unwrap the filename for {:?}", dir))?.to_string_lossy().to_string());
            return Ok(recipes);
        }
    }

    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            continue;
        }
        let mut found = list_recipes(entry.path().as_path())?;
        recipes.append(&mut found);
    }
    recipes.sort();
    Ok(recipes)
}
