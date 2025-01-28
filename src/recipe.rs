use serde::{Deserialize, Serialize};

/// Specifies how to download the source for a recipe
#[derive(Debug, Deserialize, PartialEq, Serialize)]
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

/// Specifies how to build a recipe
#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "template")]
pub enum BuildKind {
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
    Configure,
    /// Will build and install using custom commands
    #[serde(rename = "custom")]
    Custom { script: String },
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct BuildRecipe {
    #[serde(flatten)]
    pub kind: BuildKind,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct PackageRecipe {
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(rename = "shared-deps", default)]
    pub shared_deps: Vec<String>,
}

/// Everything required to build a Redox package
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Recipe {
    /// Specifies how to donload the source for this recipe
    pub source: Option<SourceRecipe>,
    /// Specifies how to build this recipe
    pub build: BuildRecipe,
    /// Specifies how to package this recipe
    #[serde(default)]
    pub package: PackageRecipe,
}

impl Recipe {
    #[inline]
    pub fn dependencies_iter(&self) -> impl Iterator<Item = &String> {
        self.build.dependencies.iter().chain(self.package.shared_deps.iter())
    }

    #[inline]
    pub fn dependencies(&self) -> Vec<String> {
        self.dependencies_iter().cloned().collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
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
                }),
                build: BuildRecipe {
                    kind: BuildKind::Cargo {
                        package_path: None,
                        cargoflags: String::new(),
                    },
                    dependencies: Vec::new(),
                },
                package: PackageRecipe {
                    dependencies: Vec::new(),
                    shared_deps: Vec::new(),
                },
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
            sha256 = "8220c0e4082fa26c07b10bfe31f641d2e33ebe1d1bb0b20221b7016bc8b78a3a"

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
                package: PackageRecipe {
                    dependencies: Vec::new(),
                    shared_deps: Vec::new(),
                },
            }
        );
    }
}
