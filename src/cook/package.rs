use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use pkg::{Package, PackageName};

use crate::{
    blake3::hash_to_hex,
    cook::{fs::*, pty::PtyOut},
    log_to_pty,
    recipe::{BuildKind, Recipe},
};

pub fn package(
    stage_dir: &Path,
    target_dir: &Path,
    name: &PackageName,
    recipe: &Recipe,
    auto_deps: &BTreeSet<PackageName>,
    logger: &PtyOut,
) -> Result<(), String> {
    if recipe.build.kind == BuildKind::None {
        // metapackages don't have stage dir
        package_toml(target_dir, name, recipe, None, auto_deps)?;
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

    let package_file = target_dir.join("stage.pkgar");
    let package_meta = target_dir.join("stage.toml");
    // Rebuild package if stage is newer
    //TODO: rebuild on recipe changes
    if package_file.is_file() {
        let stage_modified = modified_dir(stage_dir)?;
        if modified(&package_file)? < stage_modified {
            log_to_pty!(logger, "DEBUG: updating '{}'", package_file.display());
            remove_all(&package_file)?;
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

    if !package_meta.is_file() {
        package_toml(
            target_dir,
            name,
            recipe,
            Some((Path::new(public_path), &package_file)),
            auto_deps,
        )?;
    }

    Ok(())
}

pub fn package_toml(
    target_dir: &Path,
    name: &PackageName,
    recipe: &Recipe,
    package_file: Option<(&Path, &PathBuf)>,
    auto_deps: &BTreeSet<PackageName>,
) -> Result<(), String> {
    let mut depends = recipe.package.dependencies.clone();
    for dep in auto_deps.iter() {
        if !depends.contains(dep) {
            depends.push(dep.clone());
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

    let package = Package {
        name: name.clone(),
        version: package_version(recipe),
        target: redoxer::target().to_string(),
        blake3: hash,
        // this size will be different once pkgar supports compression
        network_size: size,
        storage_size: size,
        depends,
    };

    let toml_path = &target_dir.join("stage.toml");
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
