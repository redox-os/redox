use pkg::{Package, PackageName};
use std::{
    collections::{HashMap, HashSet},
    fmt::Write,
    path::PathBuf,
    str::FromStr,
};

use crate::recipe::CookRecipe;
use crate::{Error, Result};

#[derive(Clone, Copy, Debug)]
pub enum WalkTreeEntry {
    Built(u64),
    NotBuilt,
    Deduped,
    Missing,
}

impl WalkTreeEntry {
    pub fn is_valid(&self) -> bool {
        return matches!(self, WalkTreeEntry::Built(_) | WalkTreeEntry::NotBuilt);
    }
}

pub struct TreeData {
    /// list of deduped packages
    pub visited: HashSet<PackageName>,
    /// list of dependents, if requested
    pub dependents: HashMap<PackageName, HashSet<PackageName>>,
    /// total package size, only valid if tree_opt is push
    pub total_size: u64,
    /// count of packages that not exist in recipe_map
    pub total_missing: u64,
    /// count of packages that don't have package.toml
    pub total_notbuilt: u64,
    /// total packages
    pub total_count: u64,
}

pub struct TreeItem<'a> {
    /// the recipe data
    pub recipe: &'a CookRecipe,
    /// prefix for tree
    pub prefix: &'a str,
    /// last item on tree?
    pub is_last: bool,
    /// recipe status
    pub entry: WalkTreeEntry,
    /// dependencies, if not deduped
    pub dependencies: Option<&'a HashSet<&'a PackageName>>,
    /// dependents, if supplied
    pub dependents: Option<&'a HashSet<PackageName>>,
}

impl TreeData {
    pub fn new() -> Self {
        Self {
            visited: HashSet::new(),
            dependents: HashMap::new(),
            total_size: 0,
            total_count: 0,
            total_missing: 0,
            total_notbuilt: 0,
        }
    }
}

#[derive(Clone, Copy)]
pub enum DisplayOptions {
    Name,
    Path,
    Csv,
    Tree,
}

impl Default for DisplayOptions {
    fn default() -> Self {
        Self::Tree
    }
}

impl FromStr for DisplayOptions {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "name" => Ok(DisplayOptions::Name),
            "path" => Ok(DisplayOptions::Path),
            "csv" => Ok(DisplayOptions::Csv),
            "tree" => Ok(DisplayOptions::Tree),
            _ => Err(Error::Options(format!("unknown display: {s}"))),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TreeOptions {
    Repo,
    Cook,
    Push,
}

impl FromStr for TreeOptions {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "repo" => Ok(TreeOptions::Repo),
            "cook" => Ok(TreeOptions::Cook),
            "push" => Ok(TreeOptions::Push),
            _ => Err(Error::Options(format!("unknown tree: {s}"))),
        }
    }
}

pub fn display_tree_entry(
    package_roots: &[PackageName],
    recipe_map: &HashMap<&PackageName, &CookRecipe>,
    tree_opt: TreeOptions,
    display_opt: DisplayOptions,
) -> Result<TreeData> {
    let mut data = TreeData::new();
    let num_roots = package_roots.len() - 1;
    let dependents = if matches!(display_opt, DisplayOptions::Csv) {
        Some(build_dependents(recipe_map, tree_opt))
    } else {
        None
    };
    for (i, name) in package_roots.iter().enumerate() {
        walk_tree_entry(
            name,
            recipe_map,
            dependents.as_ref(),
            "",
            i == num_roots,
            tree_opt,
            &mut data,
            match display_opt {
                DisplayOptions::Name => display_name_fn,
                DisplayOptions::Path => display_path_fn,
                DisplayOptions::Csv => display_csv_fn,
                DisplayOptions::Tree => display_tree_fn,
            },
        )?;
    }
    Ok(data)
}

pub fn build_dependents(
    recipe_map: &HashMap<&PackageName, &CookRecipe>,
    tree_opt: TreeOptions,
) -> HashMap<PackageName, HashSet<PackageName>> {
    let mut dependents_map: HashMap<PackageName, HashSet<PackageName>> = HashMap::new();
    for cook_recipe in recipe_map.values() {
        let mut all_deps_set: HashSet<&PackageName> = HashSet::new();
        let pkg_meta: Package;
        match tree_opt {
            TreeOptions::Repo => {
                // no dependency recursion
            }
            TreeOptions::Cook => {
                // recursion from recipe.toml
                all_deps_set.extend(cook_recipe.recipe.build.dependencies.iter());
                all_deps_set.extend(cook_recipe.recipe.build.dev_dependencies.iter());
                all_deps_set.extend(cook_recipe.recipe.package.dependencies.iter());
            }
            TreeOptions::Push => {
                // recursion from package.toml
                if let Ok(pkg) = Package::from_file(&cook_recipe.stage_paths().2) {
                    pkg_meta = pkg;
                    all_deps_set.extend(pkg_meta.depends.iter());
                }
            }
        }
        for dep in all_deps_set {
            dependents_map
                .entry(dep.clone())
                .or_default()
                .insert(cook_recipe.name.clone());
        }
    }
    dependents_map
}

/// Does tree recursion to `op` call.
/// Tree recursion is different than recursion in [`CookRecipe`],
/// this version does call `op` from root then to their deps.
/// dependents can be precalculated with [`build_dependents`].
pub fn walk_tree_entry(
    package_name: &PackageName,
    recipe_map: &HashMap<&PackageName, &CookRecipe>,
    dependents_map: Option<&HashMap<PackageName, HashSet<PackageName>>>,
    prefix: &str,
    is_last: bool,
    tree_opt: TreeOptions,
    data: &mut TreeData,
    op: fn(TreeItem) -> Result<bool>,
) -> Result<()> {
    let cook_recipe = match recipe_map.get(package_name) {
        Some(r) => r,
        None => {
            // Data not provided, will not be processed by the build system
            op(TreeItem {
                recipe: &CookRecipe::dummy(package_name),
                prefix,
                is_last,
                entry: WalkTreeEntry::Missing,
                dependencies: None,
                dependents: None,
            })?;
            data.total_missing += 1;
            return Ok(());
        }
    };

    let (_, pkg_path, pkg_toml) = cook_recipe.stage_paths();

    let deduped = data.visited.contains(package_name);
    let pkg_meta: Package;

    let (entry, dependencies) = if deduped {
        (WalkTreeEntry::Deduped, None)
    } else {
        let entry = match (std::fs::metadata(&pkg_path), pkg_toml.is_file()) {
            (Ok(meta), _) => WalkTreeEntry::Built(meta.len()),
            (Err(_), true) => WalkTreeEntry::Built(0),
            (Err(_), false) => WalkTreeEntry::NotBuilt,
        };
        let mut all_deps_set: HashSet<&PackageName> = HashSet::new();
        match tree_opt {
            TreeOptions::Repo => {
                // no dependency recursion
            }
            TreeOptions::Cook => {
                // recursion from recipe.toml
                all_deps_set.extend(cook_recipe.recipe.build.dependencies.iter());
                all_deps_set.extend(cook_recipe.recipe.build.dev_dependencies.iter());
                all_deps_set.extend(cook_recipe.recipe.package.dependencies.iter());
            }
            TreeOptions::Push => {
                // recursion from package.toml
                if let Ok(pkg) = Package::from_file(&pkg_toml) {
                    pkg_meta = pkg;
                    all_deps_set.extend(pkg_meta.depends.iter());
                }
            }
        }
        (entry, Some(all_deps_set))
    };

    let dependents = dependents_map.and_then(|s| s.get(&cook_recipe.name));

    let cached = op(TreeItem {
        recipe: cook_recipe,
        prefix,
        is_last,
        entry,
        dependencies: dependencies.as_ref(),
        dependents,
    })?;

    if deduped || cached {
        return Ok(());
    }

    data.visited.insert(package_name.clone());
    if !cached {
        if matches!(tree_opt, TreeOptions::Push) {
            if let WalkTreeEntry::Built(pkg_size) = &entry {
                data.total_size += pkg_size;
            }
        }
        if matches!(entry, WalkTreeEntry::NotBuilt) {
            data.total_notbuilt += 1;
        }
        data.total_count += 1;
    }

    let dependencies = dependencies.unwrap();
    if dependencies.is_empty() {
        return Ok(());
    }

    let deps_count = dependencies.len();
    let child_prefix = if is_last { "    " } else { "│   " };
    for (i, dep_name) in dependencies.iter().enumerate() {
        data.dependents
            .entry((*dep_name).clone())
            .or_insert_with(|| HashSet::new())
            .insert(package_name.clone());
        walk_tree_entry(
            dep_name,
            recipe_map,
            dependents_map,
            &format!("{}{}", prefix, child_prefix),
            i == deps_count - 1,
            tree_opt,
            data,
            op,
        )?;
    }

    Ok(())
}

fn display_tree_fn(item: TreeItem) -> Result<bool> {
    let size_str = match item.entry {
        WalkTreeEntry::Built(size) => format!("[{}]", format_size(size)),
        WalkTreeEntry::NotBuilt => "(not built)".to_string(),
        WalkTreeEntry::Deduped => "".to_string(),
        WalkTreeEntry::Missing => "(omitted)".to_string(),
    };
    let is_last = item.is_last;
    let line_prefix = if is_last { "└── " } else { "├── " };
    println!(
        "{}{}{} {}",
        item.prefix, line_prefix, item.recipe.name, size_str
    );
    // TODO: check dirty build by checking source ident
    Ok(false)
}

fn display_name_fn(item: TreeItem) -> Result<bool> {
    if item.entry.is_valid() {
        println!("{}", item.recipe.name.as_str());
    }
    Ok(false)
}

fn display_path_fn(item: TreeItem) -> Result<bool> {
    if item.entry.is_valid() {
        println!("{}", item.recipe.dir.display());
    }
    Ok(false)
}

fn display_csv_fn(item: TreeItem) -> Result<bool> {
    // name, path, status, dependencies, dependents
    if item.entry.is_valid() {
        let mut deps = String::new();
        let mut depz = String::new();
        for dep in item.dependencies.unwrap() {
            if !deps.is_empty() {
                let _ = deps.write_char(';');
            }
            let _ = deps.write_str(dep.as_str());
        }
        if let Some(dependents) = item.dependents {
            for dep in dependents {
                if !depz.is_empty() {
                    let _ = depz.write_char(';');
                }
                let _ = depz.write_str(dep.as_str());
            }
        }
        println!(
            "{},{},{:?},{},{}",
            item.recipe.name.as_str(),
            item.recipe.dir.display(),
            item.entry,
            deps,
            depz,
        );
    }
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
    redox_installer::format_bytes(bytes)
}
