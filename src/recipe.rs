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

use crate::{WALK_DEPTH, cook::package as cook_package};

/// Specifies how to download the source for a recipe
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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
        /// The optional config to clone with treeless clone. Default is true if "rev" added
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
#[serde(default)]
pub struct BuildRecipe {
    #[serde(flatten)]
    pub kind: BuildKind,
    pub dependencies: Vec<PackageName>,
    #[serde(rename = "dev-dependencies")]
    pub dev_dependencies: Vec<PackageName>,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct PackageRecipe {
    pub dependencies: Vec<PackageName>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct OptionalPackageRecipe {
    pub name: String,
    pub dependencies: Vec<PackageName>,
    pub files: Vec<String>,
}

/// Everything required to build a Redox package
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct Recipe {
    /// Specifies how to download the source for this recipe
    pub source: Option<SourceRecipe>,
    /// Specifies how to build this recipe
    pub build: BuildRecipe,
    /// Specifies how to package this recipe
    pub package: PackageRecipe,
    /// Specifies optional packages based from this recipe
    #[serde(rename = "optional-packages")]
    pub optional_packages: Vec<OptionalPackageRecipe>,
}

impl BuildRecipe {
    pub fn new(kind: BuildKind) -> Self {
        let mut build = Self::default();
        build.kind = kind;
        build
    }

    pub fn set_as_remote(&mut self) {
        if self.kind == BuildKind::None {
            // BuildKind::Remote won't handle remote meta-packages
            return;
        }
        self.kind = BuildKind::Remote;
        self.dev_dependencies = Vec::new();
    }

    pub fn set_as_none(&mut self) {
        self.kind = BuildKind::None;
        self.dependencies = Vec::new();
        self.dev_dependencies = Vec::new();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CookRecipe {
    pub name: PackageName,
    pub dir: PathBuf,
    pub recipe: Recipe,
    pub target: &'static str,
    /// If false, it's listed on install config
    pub is_deps: bool,
    pub rule: String,
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

    pub fn get_packages_list(&self) -> Vec<Option<&OptionalPackageRecipe>> {
        let mut packages: Vec<Option<&OptionalPackageRecipe>> =
            self.optional_packages.iter().map(|p| Some(p)).collect();
        // the mandatory package, put last because of cook_build
        packages.push(None);
        packages
    }
}

impl CookRecipe {
    pub fn new(name: PackageName, dir: PathBuf, mut recipe: Recipe) -> Result<Self, PackageError> {
        let target = cook_package::package_target(&name);
        if name.is_host() {
            let thisname = name.name();
            let fn_map = |p: PackageName| {
                if p.is_host() {
                    if p.name() == thisname { None } else { Some(p) }
                } else {
                    Some(p.with_host())
                }
            };
            recipe.build.dependencies = recipe
                .build
                .dependencies
                .into_iter()
                .filter_map(fn_map)
                .collect();
            recipe.build.dev_dependencies = recipe
                .build
                .dev_dependencies
                .into_iter()
                .filter_map(fn_map)
                .collect();
        }
        Ok(Self {
            name,
            dir,
            recipe,
            target,
            is_deps: false,
            rule: "".into(),
        })
    }

    pub fn from_name(name: PackageName) -> Result<Self, PackageError> {
        let dir = recipes::find(name.name())
            .ok_or_else(|| PackageError::PackageNotFound(name.clone()))?;
        let file = dir.join("recipe.toml");
        let recipe = Recipe::new(&file)?;
        Self::new(name, dir.to_path_buf(), recipe)
    }

    pub fn from_list(names: Vec<PackageName>) -> Result<Vec<Self>, PackageError> {
        let mut packages = Vec::new();
        for name in names {
            packages.push(Self::from_name(name)?);
        }
        Ok(packages)
    }

    pub fn from_path(dir: &Path, read_recipe: bool, is_host: bool) -> Result<Self, PackageError> {
        let file = dir.join("recipe.toml");
        let mut name: PackageName = dir.file_name().unwrap().try_into()?;
        if is_host {
            name = name.with_host();
        }
        let recipe = if read_recipe {
            Recipe::new(&file)?
        } else {
            // clean/unfetch don't need to read recipe
            Recipe::default()
        };
        Self::new(name, dir.to_path_buf(), recipe)
    }

    fn new_recursive(
        names: &[PackageName],
        recurse_build_deps: bool,
        recurse_dev_build_deps: bool,
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
        let mut recipes_set = BTreeSet::new();
        for name in names {
            let recipe = Self::from_name(name.clone())?;

            if recurse_build_deps {
                let dependencies = Self::new_recursive(
                    &recipe.recipe.build.dependencies,
                    recurse_build_deps,
                    recurse_dev_build_deps,
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
                    if !recipes_set.contains(&dependency.name) {
                        recipes_set.insert(dependency.name.clone());
                        recipes.push(dependency);
                    }
                }
            }

            if recurse_dev_build_deps {
                let dependencies = Self::new_recursive(
                    &recipe.recipe.build.dev_dependencies,
                    recurse_build_deps,
                    recurse_dev_build_deps,
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
                    if !recipes_set.contains(&dependency.name) {
                        recipes_set.insert(dependency.name.clone());
                        recipes.push(dependency);
                    }
                }
            }

            if recurse_package_deps {
                let dependencies = Self::new_recursive(
                    &recipe.recipe.package.dependencies,
                    recurse_build_deps,
                    recurse_dev_build_deps,
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
                    if !recipes_set.contains(&dependency.name) {
                        recipes_set.insert(dependency.name.clone());
                        recipes.push(dependency);
                    }
                }
            }

            if collect_self && !recipes_set.contains(&recipe.name) {
                recipes_set.insert(recipe.name.clone());
                recipes.push(recipe);
            }
        }

        Ok(recipes)
    }

    pub fn get_build_deps_recursive(
        names: &[PackageName],
        include_dev: bool,
    ) -> Result<Vec<Self>, PackageError> {
        let packages = Self::new_recursive(
            names,
            true,
            include_dev,
            false,
            true,
            false,
            true,
            WALK_DEPTH,
        )?;

        Ok(packages)
    }

    pub fn get_package_deps_recursive(
        names: &[PackageName],
        include_names: bool,
    ) -> Result<Vec<PackageName>, PackageError> {
        // recurse_build_deps == true here as libraries (build deps) can have runtime files (package deps)
        let packages = Self::new_recursive(
            names,
            true,
            false,
            true,
            false,
            true,
            include_names,
            WALK_DEPTH,
        )?;

        Ok(packages.into_iter().map(|p| p.name).collect())
    }

    pub fn get_all_deps_names_recursive(
        names: &[PackageName],
        include_dev: bool,
    ) -> Result<Vec<PackageName>, PackageError> {
        let packages =
            Self::new_recursive(names, true, include_dev, true, true, true, true, WALK_DEPTH)?;

        Ok(packages.into_iter().map(|p| p.name).collect())
    }

    pub fn reload_recipe(&mut self) -> Result<(), PackageError> {
        self.recipe = Self::from_path(&self.dir, true, self.name.is_host())?.recipe;
        let _ = self.apply_filesystem_config(&self.rule.clone());
        Ok(())
    }

    /// returns stage dir, pkgar file and toml file.
    pub fn stage_paths(&self) -> (PathBuf, PathBuf, PathBuf) {
        let r = self.name.suffix().map(|p| OptionalPackageRecipe {
            name: p.to_string(),
            ..Default::default()
        });
        cook_package::package_stage_paths(r.as_ref(), &self.target_dir())
    }

    pub fn target_dir(&self) -> PathBuf {
        self.dir.join("target").join(self.target)
    }

    pub fn apply_filesystem_config(&mut self, rule: &str) -> Result<(), anyhow::Error> {
        match rule {
            // build from source as usual
            "source" => {}
            // keep local changes
            "local" => self.recipe.source = None,
            // download from remote build
            "binary" => {
                self.recipe.source = None;
                self.recipe.build.set_as_remote();
            }
            // don't build this recipe (unlikely to go here unless some deps need it)
            // TODO: Note that we're assuming this being ignored from e.g. metapackages
            // TODO: Will totally broke build if this recipe needed as some other build dependencies
            "ignore" => {
                self.recipe.source = None;
                self.recipe.build.set_as_none();
            }
            rule => {
                anyhow::bail!(
                    // Fail fast because we could risk losing local changes if "local" was typo'ed
                    "Invalid pkg config {} = \"{}\"\nExpecting either 'source', 'local', 'binary' or 'ignore'",
                    self.name.as_str(),
                    rule
                );
            }
        }
        self.rule = rule.to_string();

        Ok(())
    }
}

// TODO: Wrap these vectors in a struct

pub fn recipes_mark_as_deps(names: &[PackageName], packages: &mut Vec<CookRecipe>) {
    for package in packages.iter_mut() {
        package.is_deps = !names.contains(&package.name);
    }
}

pub fn recipes_flatten_package_names(packages: Vec<CookRecipe>) -> Vec<CookRecipe> {
    let mut new_packages = Vec::new();
    let mut packages_set = BTreeSet::new();
    for mut package in packages {
        let is_host = package.name.is_host();
        let mut name = package.name.with_suffix(None);
        if is_host {
            name = name.with_host();
        }
        if !packages_set.contains(name.as_str()) {
            packages_set.insert(name.to_string());
            package.name = name;
            new_packages.push(package);
        }
    }
    new_packages
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
        use crate::recipe::{BuildKind, BuildRecipe, Recipe, SourceRecipe};

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
                build: BuildRecipe::new(BuildKind::Cargo {
                    package_path: None,
                    cargoflags: String::new(),
                }),
                ..Default::default()
            }
        );
    }

    #[test]
    fn tar_custom_recipe() {
        use crate::recipe::{BuildKind, BuildRecipe, Recipe, SourceRecipe};

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
                build: BuildRecipe::new(BuildKind::Custom {
                    script: "make".to_string()
                }),
                ..Default::default()
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
                build: BuildRecipe::new(BuildKind::None),
                package: PackageRecipe {
                    dependencies: vec![PackageName::new("gcc13").unwrap()],
                    ..Default::default()
                },
                ..Default::default()
            }
        );
    }
}
