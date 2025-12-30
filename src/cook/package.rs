use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use pkg::{Package, PackageName};

use crate::{
    blake3::hash_to_hex,
    cook::{fetch, fs::*, pty::PtyOut},
    log_to_pty,
    recipe::{BuildKind, CookRecipe, OptionalPackageRecipe, Recipe},
};

pub fn package(
    recipe: &CookRecipe,
    stage_dirs: &Vec<PathBuf>,
    auto_deps: &BTreeSet<PackageName>,
    logger: &PtyOut,
) -> Result<(), String> {
    let name = &recipe.name;
    let target_dir = &recipe.target_dir();
    if recipe.recipe.build.kind == BuildKind::None {
        // metapackages don't have stage dir and optional packages
        package_toml(
            target_dir.join("stage.toml"),
            recipe,
            None,
            recipe.recipe.package.dependencies.clone(),
            &auto_deps,
        )?;
        return Ok(());
    }

    let secret_path = "build/id_ed25519.toml";
    let public_path = "build/id_ed25519.pub.toml";
    if !Path::new(secret_path).is_file() || !Path::new(public_path).is_file() {
        if !Path::new("build").is_dir() {
            create_dir(Path::new("build"))?;
        }
        let (public_key, secret_key) = pkgar_keys::SecretKeyFile::new();
        public_key
            .save(public_path)
            .map_err(|err| format!("failed to save pkgar public key: {:?}", err))?;
        secret_key
            .save(secret_path)
            .map_err(|err| format!("failed to save pkgar secret key: {:?}", err))?;
    }

    let stage_modified = modified_all(stage_dirs, modified_dir)?;

    let packages = recipe.recipe.get_packages_list();

    for package in packages {
        let (stage_dir, package_file, package_meta) = package_stage_paths(package, target_dir);
        // Rebuild package if stage is newer
        if package_file.is_file() && modified(&package_file)? < stage_modified {
            log_to_pty!(logger, "DEBUG: updating '{}'", package_file.display());
            remove_all(&package_file)?;
            if package_meta.is_file() {
                remove_all(&package_meta)?;
            }
        }

        if !package_file.is_file() {
            pkgar::create(
                secret_path,
                package_file.to_str().unwrap(),
                stage_dir.to_str().unwrap(),
            )
            .map_err(|err| format!("failed to create pkgar archive: {:?}", err))?;
        }

        let deps = if package.is_some() {
            BTreeSet::from([name.without_host()])
        } else {
            auto_deps.clone()
        };

        if !package_meta.is_file() {
            let name = match package {
                Some(p) => PackageName::new(format!("{}.{}", name.name(), p.name))
                    .map_err(|e| format!("{}", e))?,
                None => name.clone(),
            };
            let package_deps = match package {
                Some(p) => p
                    .dependencies
                    .iter()
                    .map(|dep| {
                        if dep.name().is_empty() {
                            name.with_suffix(dep.suffix())
                        } else {
                            dep.clone()
                        }
                    })
                    .collect(),
                None => recipe.recipe.package.dependencies.clone(),
            };
            package_toml(
                package_meta,
                recipe,
                Some((Path::new(public_path), &package_file)),
                package_deps,
                &deps,
            )?;
        }
    }

    Ok(())
}

pub fn package_toml(
    toml_path: PathBuf,
    recipe: &CookRecipe,
    package_file: Option<(&Path, &PathBuf)>,
    mut package_deps: Vec<PackageName>,
    auto_deps: &BTreeSet<PackageName>,
) -> Result<(), String> {
    for dep in auto_deps.iter() {
        if !package_deps.contains(dep) {
            package_deps.push(dep.clone());
        }
    }

    let (hash, size) = if let Some((pkey_path, archive_path)) = package_file {
        use pkgar_core::PackageSrc;
        let pkey = pkgar_keys::PublicKeyFile::open(pkey_path)
            .map_err(|e| format!("Unable to read public key: {e:?}"))?
            .pkey;
        let package = pkgar::PackageFile::new(archive_path, &pkey).map_err(|e| {
            format!(
                "Unable to read packaged pkgar file {}: {e:?}",
                archive_path.display(),
            )
        })?;
        let mt = std::fs::metadata(archive_path).map_err(|e| {
            format!(
                "Unable to read packaged pkgar file {}: {e:?}",
                archive_path.display(),
            )
        })?;
        (hash_to_hex(package.header().blake3), mt.len())
    } else {
        ("".into(), 0)
    };

    let ident_source = fetch::fetch_get_source_info(recipe)?;

    let package = Package {
        name: recipe.name.without_host(),
        version: package_version(&recipe.recipe),
        target: recipe.target.to_string(),
        blake3: hash,
        // this size will be different once pkgar supports compression
        network_size: size,
        storage_size: size,
        depends: package_deps,
        commit_identifier: ident_source.commit_identifier,
        source_identifier: ident_source.source_identifier,
        time_identifier: ident_source.time_identifier,
        ..Default::default()
    };

    serialize_and_write(&toml_path, &package)?;
    return Ok(());
}

fn package_version(recipe: &Recipe) -> String {
    if recipe.build.kind == BuildKind::None {
        "".into()
    } else if let Some(v) = &recipe.package.version {
        v.to_string()
    } else if let Some(r) = &recipe.source {
        if let Some(m) = r.guess_version() {
            m
        } else {
            "TODO".into()
        }
    } else {
        "TODO".into()
    }
}

pub fn package_target(name: &PackageName) -> &'static str {
    if name.is_host() {
        redoxer::host_target()
    } else {
        redoxer::target()
    }
}

pub fn package_stage_paths(
    package: Option<&OptionalPackageRecipe>,
    target_dir: &Path,
) -> (PathBuf, PathBuf, PathBuf) {
    let mut target_dir = target_dir.to_path_buf();
    if let Some(cross_target) = std::env::var("COOKBOOK_CROSS_TARGET").ok() {
        if cross_target != "" {
            // TODO: automatically pass COOKBOOK_CROSS_GNU_TARGET?
            target_dir = target_dir.join(cross_target)
        }
    }
    package_name_paths(package, &target_dir, "stage")
}

pub fn package_source_paths(
    package: Option<&OptionalPackageRecipe>,
    target_dir: &Path,
) -> (PathBuf, PathBuf, PathBuf) {
    package_name_paths(package, target_dir, "source")
}

fn package_name_paths(
    package: Option<&OptionalPackageRecipe>,
    target_dir: &Path,
    name: &str,
) -> (PathBuf, PathBuf, PathBuf) {
    let prefix_name = get_package_name(name, package);
    let package_stage = target_dir.join(&prefix_name);
    let package_file = package_stage.with_added_extension("pkgar");
    let package_meta = package_stage.with_added_extension("toml");
    (package_stage, package_file, package_meta)
}

pub fn get_package_name(name: &str, package: Option<&OptionalPackageRecipe>) -> String {
    get_package_name_inner(name, package.map(|p| p.name.as_str()))
}

fn get_package_name_inner(name: &str, package: Option<&str>) -> String {
    let mut prefix_name = name.to_string();
    if let Some(package) = package {
        prefix_name.push('.');
        prefix_name.push_str(package);
    }
    prefix_name
}
