use std::{env, fs};

use crate::recipe_find::recipe_find;

//TODO: share struct with pkgutils?
#[derive(PartialEq, serde::Deserialize, serde::Serialize)]
pub struct StageToml {
    pub name: String,
    pub version: String,
    pub target: String,
    pub depends: Vec<String>,
}

impl StageToml {
    pub fn new(name: String) -> Result<Self, String> {
        //TODO: sanitize recipe name?
        let dir = recipe_find(&name);
        if dir.is_none() {
            return Err(format!("failed to find recipe directory '{}'", name));
        }
        let dir = dir.unwrap();
        let target =
            env::var("TARGET").map_err(|err| format!("failed to read TARGET: {:?}", err))?;

        let file = dir.join("target").join(target).join("stage.toml");
        if !file.is_file() {
            return Err(format!("failed to find package file '{}'", file.display()));
        }

        let toml = fs::read_to_string(&file).map_err(|err| {
            format!(
                "failed to read package file '{}': {}\n{:#?}",
                file.display(),
                err,
                err
            )
        })?;

        toml::from_str(&toml).map_err(|err| {
            format!(
                "failed to parse package file '{}': {}\n{:#?}",
                file.display(),
                err,
                err
            )
        })
    }

    pub fn new_recursive(names: &[String], recursion: usize) -> Result<Vec<Self>, String> {
        if recursion == 0 {
            return Err(format!(
                "recursion limit while processing build dependencies: {:#?}",
                names
            ));
        }

        let mut packages = Vec::new();
        for name in names {
            let package = Self::new(name.clone())?;

            let dependencies = Self::new_recursive(&package.depends, recursion - 1)
                .map_err(|err| format!("{}: failed on loading dependencies:\n{}", name, err))?;

            for dependency in dependencies {
                if !packages.contains(&dependency) {
                    packages.push(dependency);
                }
            }

            if !packages.contains(&package) {
                packages.push(package);
            }
        }

        Ok(packages)
    }
}
