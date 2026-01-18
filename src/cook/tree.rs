use anyhow::Context;
use pkg::{Package, PackageName};
use std::fmt::Write as _;
use std::{
    collections::{HashMap, HashSet},
    fs::read_to_string,
    path::PathBuf,
};

use crate::recipe::CookRecipe;

pub enum WalkTreeEntry<'a> {
    Built(&'a PathBuf, u64),
    NotBuilt,
    Deduped,
    Missing,
}

pub fn display_tree_entry(
    package_name: &PackageName,
    recipe_map: &HashMap<&PackageName, &CookRecipe>,
    prefix: &str,
    is_last: bool,
    visited: &mut HashSet<PackageName>,
    total_size: &mut u64,
) -> anyhow::Result<()> {
    walk_tree_entry(
        package_name,
        recipe_map,
        prefix,
        is_last,
        visited,
        total_size,
        display_pkg_fn,
    )
}

pub fn walk_tree_entry(
    package_name: &PackageName,
    recipe_map: &HashMap<&PackageName, &CookRecipe>,
    prefix: &str,
    is_last: bool,
    visited: &mut HashSet<PackageName>,
    total_size: &mut u64,
    op: fn(&PackageName, &str, bool, &WalkTreeEntry) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    let cook_recipe = match recipe_map.get(package_name) {
        Some(r) => r,
        None => {
            // TODO: This is a dependency, but it's not in recipe list
            op(package_name, prefix, is_last, &WalkTreeEntry::Missing)?;
            return Ok(());
        }
    };

    let (_, pkg_path, pkg_toml) = cook_recipe.stage_paths();

    let deduped = visited.contains(package_name);
    let entry = match (std::fs::metadata(&pkg_path), deduped) {
        (_, true) => WalkTreeEntry::Deduped,
        (Ok(meta), _) => WalkTreeEntry::Built(&pkg_path, meta.len()),
        (Err(_), _) => WalkTreeEntry::NotBuilt,
    };

    op(package_name, prefix, is_last, &entry)?;

    if deduped {
        return Ok(());
    }

    visited.insert(package_name.clone());
    if let WalkTreeEntry::Built(_p, pkg_size) = &entry {
        *total_size += pkg_size;
    }
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
    let child_prefix = if is_last { "    " } else { "│   " };
    for (i, dep_name) in sorted_deps.iter().enumerate() {
        walk_tree_entry(
            dep_name,
            recipe_map,
            &format!("{}{}", prefix, child_prefix),
            i == deps_count - 1,
            visited,
            total_size,
            op,
        )?;
    }

    Ok(())
}

pub fn display_pkg_fn(
    package_name: &PackageName,
    prefix: &str,
    is_last: bool,
    entry: &WalkTreeEntry,
) -> anyhow::Result<()> {
    let size_str = match entry {
        WalkTreeEntry::Built(_path_buf, size) => format!("[{}]", format_size(*size)),
        WalkTreeEntry::NotBuilt => "(not built)".to_string(),
        WalkTreeEntry::Deduped => "".to_string(),
        WalkTreeEntry::Missing => "(dependency info missing)".to_string(),
    };
    let line_prefix = if is_last { "└── " } else { "├── " };
    println!("{}{}{} {}", prefix, line_prefix, package_name, size_str);
    Ok(())
}

pub fn walk_file_tree(dir: &PathBuf, prefix: &str, buffer: &mut String) -> std::io::Result<u64> {
    if !dir.is_dir() {
        return Ok(0);
    }
    let fmt_err = std::io::Error::other;
    let entries: Vec<_> = std::fs::read_dir(dir)?.filter_map(|e| e.ok()).collect();
    let mut total_size = 0;
    for (index, entry) in entries.iter().enumerate() {
        let path = entry.path();
        let metadata = entry.metadata()?;
        let is_last = index == entries.len() - 1;

        let line_prefix = if is_last { "└── " } else { "├── " };
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown");

        if path.is_dir() {
            writeln!(buffer, "{}{}{}/", prefix, line_prefix, file_name).map_err(fmt_err)?;
            let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
            walk_file_tree(&path, &new_prefix, buffer)?;
        } else {
            let size = metadata.len();
            total_size += size;
            writeln!(
                buffer,
                "{}{}{} ({})",
                prefix,
                line_prefix,
                file_name,
                format_size(size)
            )
            .map_err(fmt_err)?;
        }
    }

    Ok(total_size)
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
