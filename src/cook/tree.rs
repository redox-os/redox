use std::{
    collections::{HashMap, HashSet},
    fs::read_to_string,
};

use anyhow::{Context, anyhow};
use pkg::{Package, PackageName};

use crate::{cook::fs::create_target_dir, recipe::CookRecipe};

pub fn display_tree_entry(
    package_name: &PackageName,
    recipe_map: &HashMap<&PackageName, &CookRecipe>,
    prefix: &str,
    is_last: bool,
    visited: &mut HashSet<PackageName>,
    total_size: &mut u64,
) -> anyhow::Result<()> {
    let line_prefix = if is_last { "└── " } else { "├── " };
    let child_prefix = if is_last { "    " } else { "│   " };

    let cook_recipe = match recipe_map.get(package_name) {
        Some(r) => r,
        None => {
            // TODO: This is a dependency, but it's not in recipe list
            println!(
                "{}{}{} (dependency info missing)",
                prefix, line_prefix, package_name
            );
            return Ok(());
        }
    };

    let package_dir = &cook_recipe.dir;
    let pkg_path = create_target_dir(package_dir)
        .map_err(|e| anyhow!(e))?
        .join("stage.pkgar");
    let pkg_toml = create_target_dir(package_dir)
        .map_err(|e| anyhow!(e))?
        .join("stage.toml");

    let deduped = visited.contains(package_name);
    let (size_str, pkg_size) = match (std::fs::metadata(&pkg_path), deduped) {
        (_, true) => ("".to_string(), 0),
        (Ok(meta), _) => {
            let size = meta.len();
            (format!("[{}]", format_size(size)), size)
        }
        (Err(_), _) => ("(not built)".to_string(), 0),
    };

    println!("{}{}{} {}", prefix, line_prefix, package_name, size_str);

    if deduped {
        return Ok(());
    }

    visited.insert(package_name.clone());
    *total_size += pkg_size;
    let pkg_meta: Package;

    let mut all_deps_set: HashSet<&PackageName> = HashSet::new();
    if let Ok(pkg_toml_str) = read_to_string(&pkg_toml) {
        // more accurate with auto deps
        pkg_meta = toml::from_str(&pkg_toml_str)
            .context(format!("Unable to parse {}", pkg_toml.display()))?;
        all_deps_set.extend(pkg_meta.depends.iter());
    } else {
        all_deps_set.extend(cook_recipe.recipe.package.dependencies.iter());
    }

    if all_deps_set.is_empty() {
        return Ok(());
    }

    let sorted_deps: Vec<&PackageName> = all_deps_set.into_iter().collect();
    let deps_count = sorted_deps.len();
    for (i, dep_name) in sorted_deps.iter().enumerate() {
        display_tree_entry(
            dep_name,
            recipe_map,
            &format!("{}{}", prefix, child_prefix),
            i == deps_count - 1,
            visited,
            total_size,
        )?;
    }

    Ok(())
}

pub fn format_size(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let i = (bytes as f64).log(1024.0).floor() as usize;
    let size = bytes as f64 / 1024.0_f64.powi(i as i32);
    format!("{:.2} {}", size, UNITS[i])
}
