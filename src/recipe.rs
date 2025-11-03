use std::{
    collections::BTreeSet,
    convert::TryInto,
    fs,
    path::{Path, PathBuf},
};

use pkg::{PackageName, package::PackageError, recipes};
use regex::Regex;
use serde::{
    Deserialize, Serialize,
    de::{Error as DeErrorT, value::Error as DeError},
};

use crate::WALK_DEPTH;

/// Specifies how to download the source for a recipe
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum SourceRecipe {
    /// Reuse the source directory of another package
    ///
    /// This is useful when a single source repo contains multiple projects which each have their
    /// own recipe to build them.
    SameAs {
        /// Relative path to the package for which to reuse the source dir
        same_as: String,
    },
    /// Path source
    Path {
        /// The path to the source
        path: String,
    },
    /// A git repository source
    Git {
        /// The URL for the git repository, such as https://gitlab.redox-os.org/redox-os/ion.git
        git: String,
        /// The URL for an upstream repository
        upstream: Option<String>,
        /// The optional branch of the git repository to track, such as master. Please specify to
        /// make updates to the rev easier
        branch: Option<String>,
        /// The optional revision of the git repository to use for builds. Please specify for
        /// reproducible builds
        rev: Option<String>,
        /// The optional config to run as shallow fetch. Only use this for heavy git like "rust"
        /// This will disable recipe autofetching because of its cost on git server
        shallow_clone: Option<bool>,
        /// A list of patch files to apply to the source
        #[serde(default)]
        patches: Vec<String>,
        /// Optional script to run to prepare the source
        script: Option<String>,
    },
    /// A tar file source
    Tar {
        /// The URL of a tar source
        tar: String,
        /// The optional blake3 sum of the tar file. Please specify this to make reproducible
        /// builds more reliable
        blake3: Option<String>,
        /// A list of patch files to apply to the source
        #[serde(default)]
        patches: Vec<String>,
        /// Optional script to run to prepare the source, such as ./autogen.sh
        script: Option<String>,
    },
}

impl SourceRecipe {
    pub fn guess_version(&self) -> Option<String> {
        match self {
            SourceRecipe::Tar {
                tar,
                blake3: _,
                patches: _,
                script: _,
            } => {
                let re = Regex::new(r"\d+\.\d+\.\d+").unwrap();
                if let Some(arm) = re.captures(&tar) {
                    return Some(arm.get(0).unwrap().as_str().to_string());
                }
                None
            }
            _ => None,
        }
    }
}

/// Specifies how to build a recipe
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(tag = "template")]
pub enum BuildKind {
    /// Will not build (for meta packages)
    #[serde(rename = "none")]
    None,
    /// Will download compiled package from remote
    #[serde(rename = "remote")]
    Remote,
    /// Will build and install using cargo
    #[serde(rename = "cargo")]
    Cargo {
        #[serde(default)]
        package_path: Option<String>,
        #[serde(default)]
        cargoflags: String,
    },
    /// Will build and install using configure and make
    #[serde(rename = "configure")]
    Configure {
        #[serde(default)]
        configureflags: Vec<String>,
    },
    /// Will build and install using cmake
    #[serde(rename = "cmake")]
    Cmake {
        #[serde(default)]
        cmakeflags: Vec<String>,
    },
    /// Will build and install using meson
    #[serde(rename = "meson")]
    Meson {
        #[serde(default)]
        mesonflags: Vec<String>,
    },
    /// Will build and install using custom commands
    #[serde(rename = "custom")]
    Custom { script: String },
}

impl Default for BuildKind {
    fn default() -> Self {
        BuildKind::None
    }
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Serialize)]
pub struct BuildRecipe {
    #[serde(flatten, default)]
    pub kind: BuildKind,
    #[serde(default)]
    pub dependencies: Vec<PackageName>,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Serialize)]
pub struct PackageRecipe {
    #[serde(default)]
    pub dependencies: Vec<PackageName>,
    #[serde(default)]
    pub version: Option<String>,
}

/// Everything required to build a Redox package
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Serialize)]
pub struct Recipe {
    /// Specifies how to download the source for this recipe
    pub source: Option<SourceRecipe>,
    /// Specifies how to build this recipe
    #[serde(default)]
    pub build: BuildRecipe,
    /// Specifies how to package this recipe
    #[serde(default)]
    pub package: PackageRecipe,
}

impl Recipe {
    pub fn new(file: &PathBuf) -> Result<Recipe, PackageError> {
        if !file.is_file() {
            return Err(PackageError::FileMissing(file.clone()));
        }
        let toml = fs::read_to_string(&file)
            .map_err(|err| PackageError::Parse(DeError::custom(err), Some(file.clone())))?;
        let recipe: Recipe = toml::from_str(&toml)
            .map_err(|err| PackageError::Parse(DeError::custom(err), Some(file.clone())))?;
        Ok(recipe)
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct CookRecipe {
    pub name: PackageName,
    pub dir: PathBuf,
    pub recipe: Recipe,
    /// If false, it's listed on install config
    pub is_deps: bool,
}

impl CookRecipe {
    pub fn new(name: PackageName, dir: PathBuf, recipe: Recipe) -> Result<Self, PackageError> {
        Ok(Self {
            name,
            dir,
            recipe,
            is_deps: false,
        })
    }

    pub fn from_name(
        name: impl TryInto<PackageName, Error = PackageError>,
    ) -> Result<Self, PackageError> {
        let name: PackageName = name.try_into()?;
        let dir = recipes::find(name.as_str())
            .ok_or_else(|| PackageError::PackageNotFound(name.clone()))?;
        let file = dir.join("recipe.toml");
        let recipe = Recipe::new(&file)?;
        Self::new(name, dir.to_path_buf(), recipe)
    }

    pub fn from_path(dir: &Path, read_recipe: bool) -> Result<Self, PackageError> {
        let file = dir.join("recipe.toml");
        let name: PackageName = dir.file_name().unwrap().try_into()?;
        let recipe = if read_recipe {
            Recipe::new(&file)?
        } else {
            // clean/unfetch don't need to read recipe
            Recipe::default()
        };
        Self::new(name, dir.to_path_buf(), recipe)
    }

    pub fn new_recursive(
        names: &[PackageName],
        recurse_build_deps: bool,
        recurse_package_deps: bool,
        collect_build_deps: bool,
        collect_package_deps: bool,
        collect_self: bool,
        recursion: usize,
    ) -> Result<Vec<Self>, PackageError> {
        if recursion == 0 {
            return Err(PackageError::Recursion(Default::default()));
        }

        let mut recipes = Vec::new();
        for name in names {
            let recipe = Self::from_name(name.as_str())?;

            if recurse_build_deps {
                let dependencies = Self::new_recursive(
                    &recipe.recipe.build.dependencies,
                    recurse_build_deps,
                    recurse_package_deps,
                    collect_build_deps,
                    collect_package_deps,
                    collect_build_deps,
                    recursion - 1,
                )
                .map_err(|mut err| {
                    err.append_recursion(name);
                    err
                })?;

                for dependency in dependencies {
                    if !recipes.contains(&dependency) {
                        recipes.push(dependency);
                    }
                }
            }

            if recurse_package_deps {
                let dependencies = Self::new_recursive(
                    &recipe.recipe.package.dependencies,
                    recurse_build_deps,
                    recurse_package_deps,
                    collect_build_deps,
                    collect_package_deps,
                    collect_package_deps,
                    recursion - 1,
                )
                .map_err(|mut err| {
                    err.append_recursion(name);
                    err
                })?;

                for dependency in dependencies {
                    if !recipes.contains(&dependency) {
                        recipes.push(dependency);
                    }
                }
            }

            if collect_self && !recipes.contains(&recipe) {
                recipes.push(recipe);
            }
        }

        Ok(recipes)
    }

    pub fn get_build_deps_recursive(
        names: &[PackageName],
        mark_is_deps: bool,
    ) -> Result<Vec<Self>, PackageError> {
        let mut packages = Self::new_recursive(names, true, false, true, false, true, WALK_DEPTH)?;

        if mark_is_deps {
            for package in packages.iter_mut() {
                package.is_deps = !names.contains(&package.name);
            }
        }

        Ok(packages)
    }

    pub fn get_package_deps_recursive(
        names: &[PackageName],
        include_names: bool,
    ) -> Result<Vec<PackageName>, PackageError> {
        // recurse_build_deps == true here as libraries (build deps) can have runtime files (package deps)
        let packages =
            Self::new_recursive(names, true, true, false, true, include_names, WALK_DEPTH)?;

        Ok(packages.into_iter().map(|p| p.name).collect())
    }
}

#[derive(Serialize, Deserialize)]
pub struct AutoDeps {
    pub packages: BTreeSet<PackageName>,
}

#[cfg(test)]
mod tests {
    use pkg::PackageName;

    #[test]
    fn git_cargo_recipe() {
        use crate::recipe::{BuildKind, BuildRecipe, PackageRecipe, Recipe, SourceRecipe};

        let recipe: Recipe = toml::from_str(
            r#"
            [source]
            git = "https://gitlab.redox-os.org/redox-os/acid.git"
            branch = "master"
            rev = "06344744d3d55a5ac9a62a6059cb363d40699bbc"

            [build]
            template = "cargo"
        "#,
        )
        .unwrap();

        assert_eq!(
            recipe,
            Recipe {
                source: Some(SourceRecipe::Git {
                    git: "https://gitlab.redox-os.org/redox-os/acid.git".to_string(),
                    upstream: None,
                    branch: Some("master".to_string()),
                    rev: Some("06344744d3d55a5ac9a62a6059cb363d40699bbc".to_string()),
                    patches: Vec::new(),
                    script: None,
                    shallow_clone: None,
                }),
                build: BuildRecipe {
                    kind: BuildKind::Cargo {
                        package_path: None,
                        cargoflags: String::new(),
                    },
                    dependencies: Vec::new(),
                },
                package: PackageRecipe::default(),
            }
        );
    }

    #[test]
    fn tar_custom_recipe() {
        use crate::recipe::{BuildKind, BuildRecipe, PackageRecipe, Recipe, SourceRecipe};

        let recipe: Recipe = toml::from_str(
            r#"
            [source]
            tar = "http://downloads.xiph.org/releases/ogg/libogg-1.3.3.tar.xz"
            blake3 = "8220c0e4082fa26c07b10bfe31f641d2e33ebe1d1bb0b20221b7016bc8b78a3a"

            [build]
            template = "custom"
            script = "make"
        "#,
        )
        .unwrap();

        assert_eq!(
            recipe,
            Recipe {
                source: Some(SourceRecipe::Tar {
                    tar: "http://downloads.xiph.org/releases/ogg/libogg-1.3.3.tar.xz".to_string(),
                    blake3: Some(
                        "8220c0e4082fa26c07b10bfe31f641d2e33ebe1d1bb0b20221b7016bc8b78a3a"
                            .to_string()
                    ),
                    patches: Vec::new(),
                    script: None,
                }),
                build: BuildRecipe {
                    kind: BuildKind::Custom {
                        script: "make".to_string()
                    },
                    dependencies: Vec::new(),
                },
                package: PackageRecipe::default(),
            }
        );

        let source = recipe.source.unwrap();
        assert_eq!(source.guess_version(), Some("1.3.3".to_string()));
    }

    #[test]
    fn meta_recipe() {
        use crate::recipe::{BuildKind, BuildRecipe, PackageRecipe, Recipe};

        let recipe: Recipe = toml::from_str(
            r#"
            [package]
            dependencies = [
                "gcc13",
            ]
        "#,
        )
        .unwrap();

        assert_eq!(
            recipe,
            Recipe {
                source: None,
                build: BuildRecipe {
                    kind: BuildKind::None,
                    dependencies: Vec::new(),
                },
                package: PackageRecipe {
                    dependencies: vec![PackageName::new("gcc13").unwrap()],
                    version: None,
                },
            }
        );
    }
}
