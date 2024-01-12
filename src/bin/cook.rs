use cookbook::blake3::blake3_progress;
use cookbook::recipe::{Recipe, SourceRecipe, BuildKind, BuildRecipe, PackageRecipe};
use cookbook::recipe_find::recipe_find;
use std::{
    env,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{self, Command, Stdio},
    time::SystemTime,
};
use termion::{color, style};
use walkdir::{DirEntry, WalkDir};

fn remove_all(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }.map_err(|err| format!(
        "failed to remove '{}': {}\n{:?}",
        path.display(),
        err,
        err
    ))
}

fn create_dir(dir: &Path) -> Result<(), String> {
    fs::create_dir(dir).map_err(|err| format!(
        "failed to create '{}': {}\n{:?}",
        dir.display(),
        err,
        err
    ))
}

fn create_dir_clean(dir: &Path) -> Result<(), String> {
    if dir.is_dir() {
        remove_all(dir)?;
    }
    create_dir(dir)
}

fn modified(path: &Path) -> Result<SystemTime, String> {
    let metadata = fs::metadata(path).map_err(|err| format!(
        "failed to get metadata of '{}': {}\n{:#?}",
        path.display(),
        err,
        err
    ))?;
    metadata.modified().map_err(|err| format!(
        "failed to get modified time of '{}': {}\n{:#?}",
        path.display(),
        err,
        err
    ))
}

fn modified_dir_inner<F: FnMut(&DirEntry) -> bool>(dir: &Path, filter: F) -> io::Result<SystemTime> {
    let mut newest = fs::metadata(dir)?.modified()?;
    for entry_res in WalkDir::new(dir).into_iter().filter_entry(filter) {
        let entry = entry_res?;
        let modified = entry.metadata()?.modified()?;
        if modified > newest {
            newest = modified;
        }
    }
    Ok(newest)
}

fn modified_dir(dir: &Path) -> Result<SystemTime, String> {
    modified_dir_inner(dir, |_| true).map_err(|err| format!(
        "failed to get modified time of '{}': {}\n{:#?}",
        dir.display(),
        err,
        err
    ))
}

fn modified_dir_ignore_git(dir: &Path) -> Result<SystemTime, String> {
    modified_dir_inner(dir, |entry| {
        entry.file_name().to_str().map(|s| s != ".git").unwrap_or(true)
    }).map_err(|err| format!(
        "failed to get modified time of '{}': {}\n{:#?}",
        dir.display(),
        err,
        err
    ))
}

fn rename(src: &Path, dst: &Path) -> Result<(), String> {
    fs::rename(src, dst).map_err(|err| format!(
        "failed to rename '{}' to '{}': {}\n{:?}",
        src.display(),
        dst.display(),
        err,
        err
    ))
}

fn run_command(mut command: process::Command) -> Result<(), String> {
    let status = command.status().map_err(|err| format!(
        "failed to run {:?}: {}\n{:#?}",
        command,
        err,
        err
    ))?;

    if ! status.success() {
        return Err(format!(
            "failed to run {:?}: exited with status {}",
            command,
            status
        ));
    }

    Ok(())
}

fn run_command_stdin(mut command: process::Command, stdin_data: &[u8]) -> Result<(), String> {
    command.stdin(Stdio::piped());

    let mut child = command.spawn().map_err(|err| format!(
        "failed to spawn {:?}: {}\n{:#?}",
        command,
        err,
        err
    ))?;

    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(stdin_data).map_err(|err| format!(
            "failed to write stdin of {:?}: {}\n{:#?}",
            command,
            err,
            err
        ))?;
    } else {
        return Err(format!(
            "failed to find stdin of {:?}",
            command
        ));
    }

    let status = child.wait().map_err(|err| format!(
        "failed to run {:?}: {}\n{:#?}",
        command,
        err,
        err
    ))?;

    if ! status.success() {
        return Err(format!(
            "failed to run {:?}: exited with status {}",
            command,
            status
        ));
    }

    Ok(())
}

fn fetch(recipe_dir: &Path, source: &Option<SourceRecipe>) -> Result<PathBuf, String> {
    let source_dir = recipe_dir.join("source");
    match source {
        Some(SourceRecipe::Git { git, upstream, branch, rev }) => {
            //TODO: use libgit?
            if ! source_dir.is_dir() {
                // Create source.tmp
                let source_dir_tmp = recipe_dir.join("source.tmp");
                create_dir_clean(&source_dir_tmp)?;

                // Clone the repository to source.tmp
                let mut command = Command::new("git");
                command.arg("clone").arg("--recursive").arg(git);
                if let Some(branch) = branch {
                    command.arg("--branch").arg(branch);
                }
                command.arg(&source_dir_tmp);
                run_command(command)?;

                // Move source.tmp to source atomically
                rename(&source_dir_tmp, &source_dir)?;
            } else {
                // Don't let this code reset the origin for the cookbook repo
                let source_git_dir = source_dir.join(".git");
                if ! source_git_dir.is_dir() {
                    return Err(format!(
                        "'{}' is not a git repository, but recipe indicated git source",
                        source_dir.display(),
                    ));
                }

                // Reset origin
                let mut command = Command::new("git");
                command.arg("-C").arg(&source_dir);
                command.arg("remote").arg("set-url").arg("origin").arg(git);
                run_command(command)?;

                // Fetch origin
                let mut command = Command::new("git");
                command.arg("-C").arg(&source_dir);
                command.arg("fetch").arg("origin");
                run_command(command)?;
            }

            if let Some(_upstream) = upstream {
                //TODO: set upstream URL
                // git remote set-url upstream "$GIT_UPSTREAM" &> /dev/null ||
                // git remote add upstream "$GIT_UPSTREAM"
                // git fetch upstream
            }

            if let Some(rev) = rev {
                // Check out specified revision
                let mut command = Command::new("git");
                command.arg("-C").arg(&source_dir);
                command.arg("checkout").arg(rev);
                run_command(command)?;
            } else {
                //TODO: complicated stuff to check and reset branch to origin
                let mut command = Command::new("bash");
                command.arg("-c").arg(r#"
ORIGIN_BRANCH="$(git branch --remotes | grep '^  origin/HEAD -> ' | cut -d ' ' -f 5-)"
if [ -n "$BRANCH" ]
then
    ORIGIN_BRANCH="origin/$BRANCH"
fi

if [ "$(git rev-parse HEAD)" != "$(git rev-parse $ORIGIN_BRANCH)" ]
then
    git checkout -B "$(echo "$ORIGIN_BRANCH" | cut -d / -f 2-)" "$ORIGIN_BRANCH"
fi"#);
                if let Some(branch) = branch {
                    command.env("BRANCH", branch);
                }
                command.current_dir(&source_dir);
                run_command(command)?;
            }

            // Sync submodules URL
            let mut command = Command::new("git");
            command.arg("-C").arg(&source_dir);
            command.arg("submodule").arg("sync").arg("--recursive");
            run_command(command)?;

            // Update submodules
            let mut command = Command::new("git");
            command.arg("-C").arg(&source_dir);
            command.arg("submodule").arg("update").arg("--init").arg("--recursive");
            run_command(command)?;
        },
        Some(SourceRecipe::Tar { tar, blake3, patches, script }) => {
            if ! source_dir.is_dir() {
                // Download tar
                //TODO: replace wget
                let source_tar = recipe_dir.join("source.tar");
                if ! source_tar.is_file() {
                    let source_tar_tmp = recipe_dir.join("source.tar.tmp");

                    let mut command = Command::new("wget");
                    command.arg(tar);
                    command.arg("--continue").arg("-O").arg(&source_tar_tmp);
                    run_command(command)?;

                    // Move source.tar.tmp to source.tar atomically
                    rename(&source_tar_tmp, &source_tar)?;
                }

                // Calculate blake3
                let source_tar_blake3 = blake3_progress(&source_tar).map_err(|err| format!(
                    "failed to calculate blake3 of '{}': {}\n{:?}",
                    source_tar.display(),
                    err,
                    err
                ))?;
                if let Some(blake3) = blake3 {
                    // Check if it matches recipe
                    if &source_tar_blake3 != blake3 {
                        return Err(format!(
                            "calculated blake3 '{}' does not match recipe blake3 '{}'",
                            source_tar_blake3,
                            blake3
                        ));
                    }
                } else {
                    //TODO: set blake3 hash on the recipe with something like "cook fix"
                    eprintln!(
                        "WARNING: set blake3 for '{}' to '{}'",
                        source_tar.display(),
                        source_tar_blake3
                    );
                }

                // Create source.tmp
                let source_dir_tmp = recipe_dir.join("source.tmp");
                create_dir_clean(&source_dir_tmp)?;

                // Extract tar to source.tmp
                //TODO: use tar crate (how to deal with compression?)
                let mut command = Command::new("tar");
                command.arg("--extract");
                command.arg("--verbose");
                command.arg("--file").arg(&source_tar);
                command.arg("--directory").arg(&source_dir_tmp);
                command.arg("--strip-components").arg("1");
                run_command(command)?;

                // Apply patches
                for patch_name in patches {
                    let patch_file = recipe_dir.join(patch_name);
                    if ! patch_file.is_file() {
                        return Err(format!(
                            "failed to find patch file '{}'",
                            patch_file.display()
                        ));
                    }

                    let patch = fs::read_to_string(&patch_file).map_err(|err| format!(
                        "failed to read patch file '{}': {}\n{:#?}",
                        patch_file.display(),
                        err,
                        err
                    ))?;

                    let mut command = Command::new("patch");
                    command.arg("--directory").arg(&source_dir_tmp);
                    command.arg("--strip=1");
                    run_command_stdin(command, patch.as_bytes())?;
                }

                // Run source script
                if let Some(script) = script {
                    let mut command = Command::new("bash");
                    command.arg("-ex");
                    command.current_dir(&source_dir_tmp);
                    run_command_stdin(command, script.as_bytes())?;
                }

                // Move source.tmp to source atomically
                rename(&source_dir_tmp, &source_dir)?;
            }
        },
        // Local Sources
        None => {
            if ! source_dir.is_dir() {
                eprintln!(
                    "WARNING: Recipe without source section expected source dir at '{}'",
                    source_dir.display(),
                );
                create_dir(&source_dir)?;
            }
        },
    }

    Ok(source_dir)
}

fn build(recipe_dir: &Path, source_dir: &Path, target_dir: &Path, build: &BuildRecipe) -> Result<PathBuf, String> {
    let mut dep_pkgars = vec![];
    for dependency in build.dependencies.iter() {
        //TODO: sanitize name
        let dependency_dir = recipe_find(dependency, Path::new("recipes"))?;
        if dependency_dir.is_none() {
            return Err(format!(
                "failed to find recipe directory '{}'",
                dependency
            ));
        }
        dep_pkgars.push(dependency_dir.unwrap().join("target").join(redoxer::target()).join("stage.pkgar"));
    }

    let source_modified = modified_dir_ignore_git(source_dir)?;
    let deps_modified = dep_pkgars.iter().map(|pkgar| modified(pkgar)).max().unwrap_or(Ok(SystemTime::UNIX_EPOCH))?;

    let sysroot_dir = target_dir.join("sysroot");
    // Rebuild sysroot if source is newer
    //TODO: rebuild on recipe changes
    if sysroot_dir.is_dir() && (modified_dir(&sysroot_dir)? < source_modified || modified_dir(&sysroot_dir)? < deps_modified) {
        eprintln!("DEBUG: '{}' newer than '{}'", source_dir.display(), sysroot_dir.display());
        remove_all(&sysroot_dir)?;
    }
    if ! sysroot_dir.is_dir() {
        // Create sysroot.tmp
        let sysroot_dir_tmp = target_dir.join("sysroot.tmp");
        create_dir_clean(&sysroot_dir_tmp)?;

        // Make sure sysroot/include exists
        create_dir(&sysroot_dir_tmp.join("include"))?;
        // Make sure sysroot/lib exists
        create_dir(&sysroot_dir_tmp.join("lib"))?;

        for archive_path in dep_pkgars {
            let public_path = "build/id_ed25519.pub.toml";
            pkgar::extract(
                public_path,
                &archive_path,
                sysroot_dir_tmp.to_str().unwrap()
            ).map_err(|err| format!(
                "failed to install '{}' in '{}': {:?}",
                archive_path.display(),
                sysroot_dir_tmp.display(),
                err
            ))?;
        }

        // Move sysroot.tmp to sysroot atomically
        rename(&sysroot_dir_tmp, &sysroot_dir)?;
    }

    let stage_dir = target_dir.join("stage");
    // Rebuild stage if source is newer
    //TODO: rebuild on recipe changes
    if stage_dir.is_dir() && (modified_dir(&stage_dir)? < source_modified || modified_dir(&stage_dir)? < deps_modified) {
        eprintln!("DEBUG: '{}' newer than '{}'", source_dir.display(), stage_dir.display());
        remove_all(&stage_dir)?;
    }

    if ! stage_dir.is_dir() {
        // Create stage.tmp
        let stage_dir_tmp = target_dir.join("stage.tmp");
        create_dir_clean(&stage_dir_tmp)?;

        // Create build, if it does not exist
        //TODO: flag for clean builds where build is wiped out
        let build_dir = target_dir.join("build");
        if ! build_dir.is_dir() {
            create_dir_clean(&build_dir)?;
        }

        let pre_script = r#"# Common pre script
# Add cookbook bins to path
export PATH="${COOKBOOK_ROOT}/bin:${PATH}"

# This puts cargo build artifacts in the build directory
export CARGO_TARGET_DIR="${COOKBOOK_BUILD}/target"

# This adds the sysroot includes for most C compilation
#TODO: check paths for spaces!
export CFLAGS="-I${COOKBOOK_SYSROOT}/include"
export CPPFLAGS="-I${COOKBOOK_SYSROOT}/include"

# This adds the sysroot libraries and compiles binaries statically for most C compilation
#TODO: check paths for spaces!
export LDFLAGS="-L${COOKBOOK_SYSROOT}/lib --static"

# These ensure that pkg-config gets the right flags from the sysroot
export PKG_CONFIG_ALLOW_CROSS=1
export PKG_CONFIG_PATH=
export PKG_CONFIG_LIBDIR="${COOKBOOK_SYSROOT}/lib/pkgconfig"
export PKG_CONFIG_SYSROOT_DIR="${COOKBOOK_SYSROOT}"

# To build the debug version of a Cargo program, add COOKBOOK_DEBUG=true, and
# to not strip symbols from the final package, add COOKBOOK_NOSTRIP=true to the recipe
# (or to your environment) before calling cookbook_cargo or cookbook_cargo_packages
build_type=release
if [ ! -z "${COOKBOOK_DEBUG}" ]
then
    install_flags=--debug
    build_type=debug
fi

# cargo template
COOKBOOK_CARGO="${COOKBOOK_REDOXER}"
function cookbook_cargo {
    "${COOKBOOK_CARGO}" install \
        --path "${COOKBOOK_SOURCE}" \
        --root "${COOKBOOK_STAGE}/usr" \
        --locked \
        --no-track \
        ${install_flags} \
        "$@"
}

# helper for installing binaries that are cargo examples
function cookbook_cargo_examples {
    recipe="$(basename "${COOKBOOK_RECIPE}")"
    for example in "$@"
    do
        "${COOKBOOK_CARGO}" build \
            --manifest-path "${COOKBOOK_SOURCE}/Cargo.toml" \
            --example "${example}" \
            --${build_type}
        mkdir -pv "${COOKBOOK_STAGE}/usr/bin"
        cp -v \
            "target/${TARGET}/${build_type}/examples/${example}" \
            "${COOKBOOK_STAGE}/usr/bin/${recipe}_${example}"
    done
}

# helper for installing binaries that are cargo packages
function cookbook_cargo_packages {
    recipe="$(basename "${COOKBOOK_RECIPE}")"
    for package in "$@"
    do
        "${COOKBOOK_CARGO}" build \
            --manifest-path "${COOKBOOK_SOURCE}/Cargo.toml" \
            --package "${package}" \
            --${build_type}
        mkdir -pv "${COOKBOOK_STAGE}/usr/bin"
        cp -v \
            "target/${TARGET}/${build_type}/${package}" \
            "${COOKBOOK_STAGE}/usr/bin/${recipe}_${package}"
    done
}

# configure template
COOKBOOK_CONFIGURE="${COOKBOOK_SOURCE}/configure"
COOKBOOK_CONFIGURE_FLAGS=(
    --host="${TARGET}"
    --prefix=""
    --disable-shared
    --enable-static
)
COOKBOOK_MAKE="make"
COOKBOOK_MAKE_JOBS="$(nproc)"
function cookbook_configure {
    "${COOKBOOK_CONFIGURE}" "${COOKBOOK_CONFIGURE_FLAGS[@]}"
    "${COOKBOOK_MAKE}" -j "${COOKBOOK_MAKE_JOBS}"
    "${COOKBOOK_MAKE}" install DESTDIR="${COOKBOOK_STAGE}"
}
"#;

        let post_script = r#"# Common post script
# Strip binaries
if [ -d "${COOKBOOK_STAGE}/bin" ] && [ -z "${COOKBOOK_NOSTRIP}" ]
then
    find "${COOKBOOK_STAGE}/bin" -type f -exec "${TARGET}-strip" -v {} ';'
fi

if [ -d "${COOKBOOK_STAGE}/usr/bin" ] && [ -z "${COOKBOOK_NOSTRIP}" ]
then
    find "${COOKBOOK_STAGE}/usr/bin" -type f -exec "${TARGET}-strip" -v {} ';'
fi

# Remove libtool files
if [ -d "${COOKBOOK_STAGE}/lib" ]
then
    find "${COOKBOOK_STAGE}/lib" -type f -name '*.la' -exec rm -fv {} ';'
fi

if [ -d "${COOKBOOK_STAGE}/usr/lib" ]
then
    find "${COOKBOOK_STAGE}/usr/lib" -type f -name '*.la' -exec rm -fv {} ';'
fi

# Remove cargo install files
for file in .crates.toml .crates2.json
do
    if [ -f "${COOKBOOK_STAGE}/${file}" ]
    then
        rm -v "${COOKBOOK_STAGE}/${file}"
    fi
done
"#;

        //TODO: better integration with redoxer (library instead of binary)
        //TODO: configurable target
        //TODO: Add more configurability, convert scripts to Rust?
        let script = match &build.kind {
            BuildKind::Cargo => "cookbook_cargo",
            BuildKind::Configure => "cookbook_configure",
            BuildKind::Custom { script } => script
        };

        let command = {
            //TODO: remove unwraps
            let cookbook_build = build_dir.canonicalize().unwrap();
            let cookbook_recipe = recipe_dir.canonicalize().unwrap();
            let cookbook_redoxer = Path::new("target/release/cookbook_redoxer").canonicalize().unwrap();
            let cookbook_root = Path::new(".").canonicalize().unwrap();
            let cookbook_stage = stage_dir_tmp.canonicalize().unwrap();
            let cookbook_source = source_dir.canonicalize().unwrap();
            let cookbook_sysroot = sysroot_dir.canonicalize().unwrap();

            let mut command = Command::new(&cookbook_redoxer);
            command.arg("env");
            command.arg("bash").arg("-ex");
            command.current_dir(&cookbook_build);
            command.env("COOKBOOK_BUILD", &cookbook_build);
            command.env("COOKBOOK_RECIPE", &cookbook_recipe);
            command.env("COOKBOOK_REDOXER", &cookbook_redoxer);
            command.env("COOKBOOK_ROOT", &cookbook_root);
            command.env("COOKBOOK_STAGE", &cookbook_stage);
            command.env("COOKBOOK_SOURCE", &cookbook_source);
            command.env("COOKBOOK_SYSROOT", &cookbook_sysroot);
            command
        };

        let full_script = format!("{}\n{}\n{}", pre_script, script, post_script);
        run_command_stdin(command, full_script.as_bytes())?;

        // Move stage.tmp to stage atomically
        rename(&stage_dir_tmp, &stage_dir)?;
    }

    Ok(stage_dir)
}

fn package(_recipe_dir: &Path, stage_dir: &Path, target_dir: &Path, _package: &PackageRecipe) -> Result<PathBuf, String> {
    //TODO: metadata like dependencies, name, and version

    let secret_path = "build/id_ed25519.toml";
    let public_path = "build/id_ed25519.pub.toml";
    if ! Path::new(secret_path).is_file() || ! Path::new(public_path).is_file() {
        if ! Path::new("build").is_dir() {
            create_dir(Path::new("build"))?;
        }
        let (public_key, secret_key) = pkgar_keys::SecretKeyFile::new();
        public_key.save(public_path).map_err(|err| format!(
            "failed to save pkgar public key: {:?}",
            err
        ))?;
        secret_key.save(secret_path).map_err(|err| format!(
            "failed to save pkgar secret key: {:?}",
            err
        ))?;
    }

    let package_file = target_dir.join("stage.pkgar");
    // Rebuild package if stage is newer
    //TODO: rebuild on recipe changes
    if package_file.is_file() {
        let stage_modified = modified_dir(stage_dir)?;
        if modified(&package_file)? < stage_modified {
            eprintln!("DEBUG: '{}' newer than '{}'", stage_dir.display(), package_file.display());
            remove_all(&package_file)?;
        }
    }
    if ! package_file.is_file() {
        pkgar::create(
            secret_path,
            package_file.to_str().unwrap(),
            stage_dir.to_str().unwrap()
        ).map_err(|err| format!(
            "failed to create pkgar archive: {:?}",
            err
        ))?;
    }

    Ok(package_file)
}

fn cook(recipe_dir: &Path, recipe: &Recipe, fetch_only: bool) -> Result<(), String> {
    let source_dir = fetch(recipe_dir, &recipe.source).map_err(|err| format!(
        "failed to fetch: {}",
        err
    ))?;

    if fetch_only { return Ok(()); }

    let target_parent_dir = recipe_dir.join("target");
    if ! target_parent_dir.is_dir() {
        create_dir(&target_parent_dir)?;
    }
    let target_dir = target_parent_dir.join(redoxer::target());
    if ! target_dir.is_dir() {
        create_dir(&target_dir)?;
    }

    let stage_dir = build(recipe_dir, &source_dir, &target_dir, &recipe.build).map_err(|err| format!(
        "failed to build: {}",
        err
    ))?;

    let _package_file = package(recipe_dir, &stage_dir, &target_dir, &recipe.package).map_err(|err| format!(
        "failed to package: {}",
        err
    ))?;

    Ok(())
}

pub struct CookRecipe {
    name: String,
    dir: PathBuf,
    recipe: Recipe,
}

impl CookRecipe {
    pub fn new(name: String) -> Result<Self, String> {
        //TODO: sanitize recipe name?
        let dir = recipe_find(&name, Path::new("recipes"))?;
        if dir.is_none() {
            return Err(format!(
                "failed to find recipe directory '{}'",
                name
            ));
        }
        let dir = dir.unwrap();
        let file = dir.join("recipe.toml");
        if ! file.is_file() {
            return Err(format!(
                "failed to find recipe file '{}'",
                file.display()
            ));
        }

        let toml = fs::read_to_string(&file).map_err(|err| format!(
            "failed to read recipe file '{}': {}\n{:#?}",
            file.display(),
            err,
            err
        ))?;

        let recipe: Recipe = toml::from_str(&toml).map_err(|err| format!(
            "failed to parse recipe file '{}': {}\n{:#?}",
            file.display(),
            err,
            err
        ))?;

        Ok(Self {
            name,
            dir,
            recipe
        })
    }

    //TODO: make this more efficient, smarter, and not return duplicates
    pub fn new_recursive(names: &[String], recursion: usize) -> Result<Vec<Self>, String> {
        if recursion == 0 {
            return Err(format!(
                "recursion limit while processing build dependencies: {:#?}",
                names
            ));
        }

        let mut recipes = Vec::new();
        for name in names {
            let recipe = Self::new(name.clone())?;

            let dependencies = Self::new_recursive(
                &recipe.recipe.build.dependencies,
                recursion - 1
            ).map_err(|err| format!(
                "{}: failed on loading build dependencies:\n{}",
                name,
                err
            ))?;

            for dependency in dependencies {
                recipes.push(dependency);
            }

            recipes.push(recipe);
        }

        Ok(recipes)
    }
}

fn main() {
    let mut matching = true;
    let mut dry_run = false;
    let mut fetch_only = false;
    let mut quiet = false;
    let mut recipe_names = Vec::new();
    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--" if matching => matching = false,
            "-d" | "--dry-run" if matching => dry_run = true,
            "--fetch-only" if matching => fetch_only = true,
            "-q" | "--quiet" if matching => quiet = true,
            _ => recipe_names.push(arg),
        }
    }

    let recipes = match CookRecipe::new_recursive(&recipe_names, 16) {
        Ok(ok) => ok,
        Err(err) => {
            eprintln!(
                "{}{}cook - error:{}{} {}",
                style::Bold,
                color::Fg(color::AnsiValue(196)),
                color::Fg(color::Reset),
                style::Reset,
                err,
            );
            process::exit(1);
        }
    };

    for recipe in recipes {
        if ! quiet {
            eprintln!(
                "{}{}cook - {}{}{}",
                style::Bold,
                color::Fg(color::AnsiValue(215)),
                recipe.name,
                color::Fg(color::Reset),
                style::Reset,
            );
        }

        let res = if dry_run {
            if ! quiet {
                eprintln!("DRY RUN: {:#?}", recipe.recipe);
            }
            Ok(())
        } else {
            cook(&recipe.dir, &recipe.recipe, fetch_only)
        };

        match res {
            Ok(()) => {
                if ! quiet {
                    eprintln!(
                        "{}{}cook - {} - successful{}{}",
                        style::Bold,
                        color::Fg(color::AnsiValue(46)),
                        recipe.name,
                        color::Fg(color::Reset),
                        style::Reset,
                    );
                }
            },
            Err(err) => {
                eprintln!(
                    "{}{}cook - {} - error:{}{} {}",
                    style::Bold,
                    color::Fg(color::AnsiValue(196)),
                    recipe.name,
                    color::Fg(color::Reset),
                    style::Reset,
                    err,
                );
                process::exit(1);
            }
        }
    }
}
