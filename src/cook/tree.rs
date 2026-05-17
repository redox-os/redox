use pkg::{Package, PackageName};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use crate::Result;
use crate::cook::fs;
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
    is_build_tree: bool,
    visited: &mut HashSet<PackageName>,
    total_size: &mut u64,
    total_count: &mut u64,
) -> Result<()> {
    walk_tree_entry(
        package_name,
        recipe_map,
        prefix,
        is_last,
        is_build_tree,
        visited,
        total_size,
        total_count,
        display_pkg_fn,
    )
}

pub fn walk_tree_entry(
    package_name: &PackageName,
    recipe_map: &HashMap<&PackageName, &CookRecipe>,
    prefix: &str,
    is_last: bool,
    is_build_tree: bool,
    visited: &mut HashSet<PackageName>,
    total_size: &mut u64,
    total_count: &mut u64,
    op: fn(&PackageName, &str, bool, &WalkTreeEntry) -> Result<bool>,
) -> Result<()> {
    let cook_recipe = match recipe_map.get(package_name) {
        Some(r) => r,
        None => {
            // Data not provided, will not be processed by the build system
            op(package_name, prefix, is_last, &WalkTreeEntry::Missing)?;
            return Ok(());
        }
    };

    let (_, pkg_path, pkg_toml) = cook_recipe.stage_paths();

    let deduped = visited.contains(package_name);
    let entry = if deduped {
        WalkTreeEntry::Deduped
    } else {
        match (std::fs::metadata(&pkg_path), pkg_toml.is_file()) {
            (Ok(meta), _) => WalkTreeEntry::Built(&pkg_path, meta.len()),
            (Err(_), true) => WalkTreeEntry::Built(&pkg_path, 0),
            (Err(_), false) => WalkTreeEntry::NotBuilt,
        }
    };

    let cached = op(package_name, prefix, is_last, &entry)?;

    if deduped || cached {
        return Ok(());
    }

    visited.insert(package_name.clone());
    if !cached {
        if is_build_tree {
            if matches!(entry, WalkTreeEntry::NotBuilt) {
                *total_size += 1;
            }
        } else {
            if let WalkTreeEntry::Built(_p, pkg_size) = &entry {
                *total_size += pkg_size;
            }
        }
        *total_count += 1;
    }
    let pkg_meta: Package;

    let mut all_deps_set: HashSet<&PackageName> = HashSet::new();
    if is_build_tree {
        all_deps_set.extend(cook_recipe.recipe.build.dependencies.iter());
        all_deps_set.extend(cook_recipe.recipe.package.dependencies.iter());
    } else {
        if let Ok(pkg_toml_str) = fs::read_to_string(&pkg_toml) {
            // more accurate with auto deps
            pkg_meta = Package::from_toml(&pkg_toml_str)?;
            all_deps_set.extend(pkg_meta.depends.iter());
        }
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
            is_build_tree,
            visited,
            total_size,
            total_count,
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
) -> Result<bool> {
    let size_str = match entry {
        WalkTreeEntry::Built(_path_buf, size) => format!("[{}]", format_size(*size)),
        WalkTreeEntry::NotBuilt => "(not built)".to_string(),
        WalkTreeEntry::Deduped => "".to_string(),
        WalkTreeEntry::Missing => "(omitted)".to_string(),
    };
    let line_prefix = if is_last { "└── " } else { "├── " };
    println!("{}{}{} {}", prefix, line_prefix, package_name, size_str);
    // TODO: check dirty build by checking source ident
    Ok(false)
}

pub fn walk_file_tree(
    dir: &PathBuf,
    prefix: &str,
    buffer: &mut Vec<String>,
) -> std::io::Result<u64> {
    if !dir.is_dir() {
        return Ok(0);
    }
    let mut entries: Vec<_> = std::fs::read_dir(dir)?.filter_map(|e| e.ok()).collect();
    entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
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

        if metadata.is_dir() {
            buffer.push(format!("{}{}{}/", prefix, line_prefix, file_name));
            let last_len = buffer.len();
            let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
            total_size += walk_file_tree(&path, &new_prefix, buffer)?;
            if buffer.len() == last_len {
                // pkgar doesn't capture empty directory
                buffer.pop();
            }
        } else if metadata.is_symlink() {
            let size = metadata.len();
            total_size += size;
            buffer.push(format!(
                "{}{}{} -> {:?}",
                prefix,
                line_prefix,
                file_name,
                std::fs::read_link(&path)?.display()
            ));
        } else {
            let size = metadata.len();
            total_size += size;
            buffer.push(format!(
                "{}{}{} ({})",
                prefix,
                line_prefix,
                file_name,
                format_size(size)
            ));
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
