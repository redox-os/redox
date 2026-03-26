use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use pkg::{Package, PackageError, PackageName};

// This file contains code that caches recipe paths.

// TODO: This file is previously resides in `pkg` crate,
// and can actually be merged with other logic in this cookbook.

static RECIPE_PATHS: LazyLock<HashMap<PackageName, PathBuf>> = LazyLock::new(|| {
    let mut recipe_paths = HashMap::new();
    for entry_res in ignore::Walk::new("recipes") {
        let Ok(entry) = entry_res else {
            continue;
        };
        if entry.file_name() == OsStr::new("recipe.toml") {
            let recipe_file = entry.path();
            let Some(recipe_dir) = recipe_file.parent() else {
                continue;
            };
            let Some(recipe_name) = recipe_dir
                .file_name()
                .and_then(|x| x.to_str()?.try_into().ok())
            else {
                continue;
            };
            if let Some(other_dir) = recipe_paths.insert(recipe_name, recipe_dir.to_path_buf()) {
                eprintln!(
                    "recipe {:?} has two or more entries: {:?} replaced by {:?}",
                    recipe_dir.file_name(),
                    other_dir,
                    recipe_dir,
                );
            }
        }
    }
    recipe_paths
});

pub fn find(recipe: &str) -> Option<&'static Path> {
    RECIPE_PATHS.get(recipe).map(PathBuf::as_path)
}

pub fn list(prefix: impl AsRef<Path>) -> BTreeSet<PathBuf> {
    let prefix = prefix.as_ref();
    RECIPE_PATHS
        .values()
        .map(|path| prefix.join(path))
        .collect()
}

pub fn new(name: &PackageName) -> Result<Package, PackageError> {
    let dir = find(name.name()).ok_or_else(|| PackageError::PackageNotFound(name.clone()))?;
    from_path(dir, name.suffix())
}

pub fn from_path(dir: &Path, feature: Option<&str>) -> Result<Package, PackageError> {
    let target = redoxer::target();

    let stage_name = match feature {
        Some(f) => Cow::Owned(format!("stage.{f}.toml")),
        None => Cow::Borrowed("stage.toml"),
    };

    let file = dir.join("target").join(target).join(stage_name.as_ref());
    if !file.is_file() {
        return Err(PackageError::FileMissing(file));
    }

    let toml = std::fs::read_to_string(&file)
        .map_err(|err| PackageError::FileError(err.raw_os_error(), file.clone()))?;
    toml::from_str(&toml).map_err(|err| PackageError::Parse(err, Some(file)))
}

pub fn new_recursive(
    names: &[PackageName],
    nonstop: bool,
    recursion: usize,
) -> Result<Vec<Package>, PackageError> {
    if names.len() == 0 {
        return Ok(vec![]);
    }
    let (list, map) = new_recursive_nonstop(names, recursion);
    if nonstop && list.len() > 0 {
        Ok(list)
    } else if !nonstop && map.len() == list.len() {
        Ok(list)
    } else {
        let (_, res) = map.into_iter().find(|(_, v)| v.is_err()).unwrap();
        Err(res.err().unwrap())
    }
}

// list ordered success packages and map of failed packages
// a package can be both success and failed if dependencies aren't satistied
pub fn new_recursive_nonstop(
    names: &[PackageName],
    recursion: usize,
) -> (
    Vec<Package>,
    BTreeMap<PackageName, Result<(), PackageError>>,
) {
    let mut packages = Vec::new();
    let mut packages_map = BTreeMap::new();
    for name in names {
        if packages_map.contains_key(name) {
            continue;
        }

        let package = if recursion == 0 {
            Err(PackageError::Recursion(Default::default()))
        } else {
            new(name)
        };

        match package {
            Ok(package) => {
                let mut has_invalid_dependency = false;
                let (dependencies, dependencies_map) =
                    new_recursive_nonstop(&package.depends, recursion - 1);
                for dependency in dependencies {
                    if !packages_map.contains_key(&dependency.name) {
                        packages_map.insert(dependency.name.clone(), Ok(()));
                        packages.push(dependency);
                    }
                }
                for (dep_name, result) in dependencies_map {
                    if let Err(mut e) = result {
                        if !packages_map.contains_key(&dep_name) {
                            e.append_recursion(name);
                            packages_map.insert(dep_name, Err(e));
                        }
                        has_invalid_dependency = true;
                    }
                }
                // TODO: this if check is redundant
                if !packages_map.contains_key(name) {
                    packages_map.insert(
                        name.clone(),
                        if has_invalid_dependency {
                            Err(PackageError::DependencyInvalid(name.clone()))
                        } else {
                            Ok(())
                        },
                    );
                    packages.push(package);
                }
            }
            Err(e) => {
                packages_map.insert(name.clone(), Err(e));
            }
        }
    }

    (packages, packages_map)
}
