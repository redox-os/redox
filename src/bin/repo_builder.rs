use anyhow::anyhow;
use cookbook::WALK_DEPTH;
use cookbook::config::{get_config, init_config};
use cookbook::cook::package as cook_package;
use cookbook::recipe::CookRecipe;
use pkg::{Package, PackageName, recipes};
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

#[derive(Clone)]
struct CliConfig {
    repo_dir: PathBuf,
    nonstop: bool,
    appstream: bool,
    recipe_list: Vec<String>,
}

impl CliConfig {
    fn parse_args() -> Result<Self, std::io::Error> {
        let mut args = env::args().skip(1);
        let repo_dir = args
            .next()
            .expect("Usage: repo_builder <REPO_DIR> <recipe1> <recipe2> ...");
        Ok(CliConfig {
            repo_dir: PathBuf::from(repo_dir),
            nonstop: get_config().cook.nonstop,
            appstream: env::var("COOKBOOK_APPSTREAM").ok().as_deref() == Some("true"),
            recipe_list: args.collect(),
        })
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_config();
    let conf = CliConfig::parse_args()?;
    Ok(publish_packages(&conf)?)
}

// TODO: Make this callable from repo bin
fn publish_packages(config: &CliConfig) -> anyhow::Result<()> {
    let repo_path = &config.repo_dir.join(redoxer::target());
    if !repo_path.is_dir() {
        fs::create_dir_all(repo_path)?;
    }

    // Runtime dependencies include both `[package.dependencies]` and dynamically
    // linked packages discovered by auto_deps.
    //
    // The following adds the package dependencies of the recipes to the repo as
    // well.
    let recipe_list = Package::new_recursive(
        &config
            .recipe_list
            .iter()
            .map(PackageName::new)
            // Don't publish host packages
            .filter(|pkg| pkg.as_ref().is_ok_and(|p| !p.is_host()))
            .collect::<Result<Vec<_>, _>>()?,
        config.nonstop,
        WALK_DEPTH,
    )?
    .into_iter()
    .map(|pkg| pkg.name.clone())
    .collect::<Vec<_>>();

    let mut appstream_sources: HashMap<String, PathBuf> = HashMap::new();
    let mut packages: BTreeMap<String, String> = BTreeMap::new();

    // === 1. Push recipes in list ===
    for recipe in &recipe_list {
        let Some(recipe_path) = recipes::find(recipe.name()) else {
            eprintln!("recipe {} not found", recipe);
            continue;
        };
        let Ok(cookbook_recipe) = CookRecipe::from_path(recipe_path, true, false) else {
            eprintln!("recipe {} unable to read", recipe);
            continue;
        };

        let target_dir = cookbook_recipe.target_dir();
        for package in cookbook_recipe.recipe.get_packages_list() {
            let (stage_dir, pkgar_src, toml_src) =
                cook_package::package_stage_paths(package, &target_dir);
            let recipe_name = cook_package::get_package_name(recipe.name(), package);
            let pkgar_dst = repo_path.join(format!("{}.pkgar", recipe_name));
            let toml_dst = repo_path.join(format!("{}.toml", recipe_name));

            if !fs::exists(&toml_src)? {
                eprintln!("recipe {} is missing stage.toml", recipe_name);
                continue;
            }

            if is_newer(&toml_src, &toml_dst) {
                eprintln!("\x1b[01;38;5;155mrepo - publishing {}\x1b[0m", recipe_name);
                if fs::exists(&pkgar_src)? {
                    fs::copy(&pkgar_src, &pkgar_dst)?;
                }
                fs::copy(&toml_src, &toml_dst)?;
            }

            if stage_dir.join("usr/share/metainfo").exists() {
                appstream_sources.insert(recipe.name().to_string(), stage_dir.clone());
            }
        }
    }

    // === 2. Optional AppStream generation ===
    if config.appstream {
        eprintln!("\x1b[01;38;5;155mrepo - generating appstream data\x1b[0m");

        let root = env::var("ROOT").unwrap_or_else(|_| ".".into());
        let target = env::var("TARGET").unwrap_or_else(|_| "x86_64-unknown-linux-gnu".into());
        let appstream_root = Path::new(&root)
            .join("build")
            .join(&target)
            .join("appstream");
        let appstream_pkg = repo_path.join("repo-appstream.pkgar");

        fs::remove_dir_all(&appstream_root).ok();
        fs::remove_file(&appstream_pkg).ok();
        fs::create_dir_all(&appstream_root)?;

        if !appstream_sources.is_empty() {
            let mut compose_cmd = Command::new("appstreamcli");
            compose_cmd
                .arg("compose")
                .arg("--origin=pkgar")
                .arg("--print-report=full")
                .arg(format!("--result-root={}", appstream_root.display()));

            for (_recipe, source_path) in &appstream_sources {
                compose_cmd.arg(source_path);
            }

            compose_cmd
                .status()?
                .success()
                .then_some(())
                .ok_or(anyhow!("appstreamcli failed"))?;

            pkgar::create(
                format!("{}/build/id_ed25519.toml", root),
                &appstream_pkg,
                &appstream_root,
            )?;
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

        let empty_ver = Value::String("".to_string());
        let version_str = parsed
            .get("blake3")
            .unwrap_or_else(|| parsed.get("version").unwrap_or_else(|| &empty_ver))
            .to_string(); // includes quotes
        let package_name = path.file_stem().unwrap().to_string_lossy().to_string();
        packages.insert(package_name, version_str);
    }

    // FIXME: Use proper TOML serializer
    let mut output = String::from("[packages]\n");
    for (name, version) in &packages {
        output.push_str(&if name.contains('.') {
            format!("\"{name}\" = {version}\n")
        } else {
            format!("{name} = {version}\n")
        });
    }

    let mut output_file = File::create(&repo_toml_path)?;
    output_file.write_all(output.as_bytes())?;

    Ok(())
}
