use pkg::recipes;
use std::collections::{BTreeMap, HashMap};
use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use toml::Value;

fn is_newer(src: &Path, dst: &Path) -> bool {
    match (fs::metadata(src), fs::metadata(dst)) {
        (Ok(src_meta), Ok(dst_meta)) => match (src_meta.modified(), dst_meta.modified()) {
            (Ok(src_time), Ok(dst_time)) => src_time > dst_time,
            (Ok(_), Err(_)) => true,
            _ => false,
        },
        (Ok(_), Err(_)) => true,
        _ => false,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let repo_dir = args
        .next()
        .expect("Usage: repo_builder <REPO_DIR> <recipe1> <recipe2> ...");
    let recipe_list: Vec<String> = args.collect();
    let repo_path = Path::new(&repo_dir);

    let mut appstream_sources: HashMap<String, PathBuf> = HashMap::new();
    let mut packages: BTreeMap<String, String> = BTreeMap::new();

    // === 1. Push recipes in list ===
    for recipe in &recipe_list {
        let Some(recipe_path) = recipes::find(recipe) else {
            eprintln!("recipe {} not found", recipe);
            continue;
        };

        let cookbook_recipe = Path::new(&recipe_path);
        let target = env::var("TARGET").unwrap_or_else(|_| "x86_64-unknown-linux-gnu".into());
        let stage_dir = cookbook_recipe.join("target").join(&target).join("stage");

        let pkgar_src = stage_dir.with_extension("pkgar");
        let pkgar_dst = repo_path.join(format!("{}.pkgar", recipe));
        let toml_src = stage_dir.with_extension("toml");
        let toml_dst = repo_path.join(format!("{}.toml", recipe));

        if is_newer(&toml_src, &toml_dst) {
            eprintln!("\x1b[01;38;5;155mrepo - publishing {}\x1b[0m", recipe);
            if fs::exists(&pkgar_src)? {
                fs::copy(&pkgar_src, &pkgar_dst)?;
            }
            fs::copy(&toml_src, &toml_dst)?;
        }

        if stage_dir.join("usr/share/metainfo").exists() {
            appstream_sources.insert(recipe.clone(), stage_dir.clone());
        }
    }

    // === 2. Optional AppStream generation ===
    if env::var("APPSTREAM").ok().as_deref() == Some("1") {
        eprintln!("\x1b[01;38;5;155mrepo - generating appstream data\x1b[0m");

        let root = env::var("ROOT").unwrap_or_else(|_| ".".into());
        let target = env::var("TARGET").unwrap_or_else(|_| "x86_64-unknown-linux-gnu".into());
        let appstream_root = Path::new(&root)
            .join("build")
            .join(&target)
            .join("appstream");
        let appstream_pkg = repo_path.join("appstream.pkgar");

        fs::remove_dir_all(&appstream_root).ok();
        fs::remove_file(&appstream_pkg).ok();
        fs::create_dir_all(&appstream_root)?;

        if !appstream_sources.is_empty() {
            let mut compose_cmd = Command::new("appstreamcli");
            compose_cmd
                .arg("compose")
                .arg("--origin=pkgar")
                .arg(format!("--result-root={}", appstream_root.display()));

            for (_recipe, source_path) in &appstream_sources {
                compose_cmd.arg(source_path);
            }

            compose_cmd
                .status()?
                .success()
                .then_some(())
                .ok_or("appstreamcli failed")?;

            Command::new("pkgar")
                .arg("create")
                .arg("--archive")
                .arg(&appstream_pkg)
                .arg("--skey")
                .arg(format!("{}/build/id_ed25519.toml", root))
                .arg(&appstream_root)
                .status()?
                .success()
                .then_some(())
                .ok_or("pkgar create failed")?;
        }
    }

    eprintln!("\x1b[01;38;5;155mrepo - generating repo.toml\x1b[0m");

    // === 3. Read and update repo.toml ===
    let repo_toml_path = repo_path.join("repo.toml");
    if repo_toml_path.exists() {
        let mut file = File::open(&repo_toml_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let parsed: Value = toml::from_str(&contents)?;
        if let Some(pkg_table) = parsed.get("packages").and_then(|v| v.as_table()) {
            for (k, v) in pkg_table {
                if let Some(s) = v.as_str() {
                    packages.insert(k.clone(), format!("\"{}\"", s));
                } else {
                    packages.insert(k.clone(), v.to_string());
                }
            }
        }
    }

    for entry in fs::read_dir(&repo_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("toml") {
            continue;
        }

        if path.file_stem().and_then(|s| s.to_str()) == Some("repo") {
            continue;
        }

        let content = fs::read_to_string(&path)?;
        let parsed: Value = toml::from_str(&content)?;

        if let Some(version_val) = parsed.get("version") {
            let version_str = version_val.to_string(); // includes quotes
            let package_name = path.file_stem().unwrap().to_string_lossy().to_string();
            packages.insert(package_name, version_str);
        } else {
            eprintln!("Warning: no [version] found in {:?}", path);
        }
    }

    // FIXME: Use proper TOML serializer
    let mut output = String::from("[packages]\n");
    for (name, version) in &packages {
        output.push_str(&format!("{name} = {version}\n"));
    }

    let mut output_file = File::create(&repo_toml_path)?;
    output_file.write_all(output.as_bytes())?;

    Ok(())
}
