use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use pkg::{Package, PackageName};
use redoxer::target;

use crate::{REMOTE_PKG_SOURCE, cook::fs::*, recipe::AutoDeps};

fn get_remote_url(name: &PackageName, ext: &str) -> String {
    return format!("{}/{}/{}.{}", REMOTE_PKG_SOURCE, target(), name, ext);
}
fn get_pubkey_url() -> String {
    return format!("{}/id_ed25519.pub.toml", REMOTE_PKG_SOURCE);
}

pub fn build_remote(
    target_dir: &Path,
    name: &PackageName,
    offline_mode: bool,
) -> Result<(PathBuf, BTreeSet<PackageName>), String> {
    // download straight from remote source then declare pkg dependencies as autodeps dependency
    let stage_dir = target_dir.join("stage");

    let source_pkgar = target_dir.join("source.pkgar");
    let source_toml = target_dir.join("source.toml");
    let source_pubkey = target_dir.join("id_ed25519.pub.toml");

    if !offline_mode {
        download_wget(&get_remote_url(name, "pkgar"), &source_pkgar)?;
        download_wget(&get_remote_url(name, "toml"), &source_toml)?;
        download_wget(&get_pubkey_url(), &source_pubkey)?;
    } else {
        offline_check_exists(&source_pkgar)?;
        offline_check_exists(&source_toml)?;
        offline_check_exists(&source_pubkey)?;
    }

    if stage_dir.is_dir() && modified(&source_pkgar)? > modified(&stage_dir)? {
        remove_all(&stage_dir)?
    }
    if !stage_dir.is_dir() {
        let stage_dir_tmp = target_dir.join("stage.tmp");

        pkgar::extract(&source_pubkey, &source_pkgar, &stage_dir_tmp).map_err(|err| {
            format!(
                "failed to install '{}' in '{}': {:?}",
                source_pkgar.display(),
                stage_dir_tmp.display(),
                err
            )
        })?;

        // Move stage.tmp to stage atomically
        rename(&stage_dir_tmp, &stage_dir)?;
    }

    let auto_deps_path = target_dir.join("auto_deps.toml");
    if auto_deps_path.is_file() && modified(&auto_deps_path)? < modified(&stage_dir)? {
        remove_all(&auto_deps_path)?
    }

    let auto_deps = if auto_deps_path.exists() {
        let toml_content =
            fs::read_to_string(&auto_deps_path).map_err(|_| "failed to read cached auto_deps")?;
        let wrapper: AutoDeps =
            toml::from_str(&toml_content).map_err(|_| "failed to deserialize cached auto_deps")?;
        wrapper.packages
    } else {
        let toml_content =
            fs::read_to_string(&source_toml).map_err(|_| "failed to read source.toml")?;
        let pkg_toml: Package =
            toml::from_str(&toml_content).map_err(|_| "failed to deserialize source.toml")?;
        let wrapper = AutoDeps {
            packages: pkg_toml.depends.into_iter().collect(),
        };
        serialize_and_write(&auto_deps_path, &wrapper)?;
        wrapper.packages
    };

    Ok((stage_dir, auto_deps))
}
