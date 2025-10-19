use cookbook::config::init_config;
use cookbook::cook::build::build_remote;
use cookbook::cook::fetch::*;
use cookbook::cook::fs::*;
use cookbook::cook::script::SHARED_PRESCRIPT;
use cookbook::recipe::{AutoDeps, BuildKind, CookRecipe, Recipe};
use pkg::package::Package;
use pkg::{PackageName, recipes};
use std::collections::VecDeque;
use std::convert::TryInto;
use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
    process::{self, Command},
    str,
    time::SystemTime,
};
use termion::{color, style};

use cookbook::{WALK_DEPTH, is_redox};

fn auto_deps(
    stage_dir: &Path,
    dep_pkgars: &BTreeSet<(PackageName, PathBuf)>,
) -> BTreeSet<PackageName> {
    let mut paths = BTreeSet::new();
    let mut visited = BTreeSet::new();
    // Base directories may need to be updated for packages that place binaries in odd locations.
    let mut walk = VecDeque::from([
        stage_dir.join("libexec"),
        stage_dir.join("usr/bin"),
        stage_dir.join("usr/games"),
        stage_dir.join("usr/lib"),
        stage_dir.join("usr/libexec"),
    ]);

    // Recursively (DFS) walk each directory to ensure nested libs and bins are checked.
    while let Some(dir) = walk.pop_front() {
        let Ok(dir) = dir.canonicalize() else {
            continue;
        };
        if visited.contains(&dir) {
            #[cfg(debug_assertions)]
            eprintln!("DEBUG: auto_deps => Skipping `{dir:?}` (already visited)");
            continue;
        }
        assert!(
            visited.insert(dir.clone()),
            "Directory `{:?}` should not be in visited\nVisited: {:#?}",
            dir,
            visited
        );

        let Ok(read_dir) = fs::read_dir(&dir) else {
            continue;
        };
        for entry_res in read_dir {
            let Ok(entry) = entry_res else { continue };
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_file() {
                paths.insert(entry.path());
            } else if file_type.is_dir() {
                walk.push_front(entry.path());
            }
        }
    }

    let mut needed = BTreeSet::new();
    for path in paths {
        let Ok(file) = fs::File::open(&path) else {
            continue;
        };
        let read_cache = object::ReadCache::new(file);
        let Ok(object) = object::build::elf::Builder::read(&read_cache) else {
            continue;
        };
        let Some(dynamic_data) = object.dynamic_data() else {
            continue;
        };
        for dynamic in dynamic_data {
            let object::build::elf::Dynamic::String { tag, val } = dynamic else {
                continue;
            };
            if *tag == object::elf::DT_NEEDED {
                let Ok(name) = str::from_utf8(val) else {
                    continue;
                };
                if let Ok(relative_path) = path.strip_prefix(stage_dir) {
                    eprintln!("DEBUG: {} needs {}", relative_path.display(), name);
                }
                needed.insert(name.to_string());
            }
        }
    }

    let mut missing = needed.clone();
    // relibc and friends will always be installed
    for preinstalled in &["libc.so.6", "libgcc_s.so.1", "libstdc++.so.6"] {
        missing.remove(*preinstalled);
    }

    let mut deps = BTreeSet::new();
    if let Ok(key_file) = pkgar_keys::PublicKeyFile::open("build/id_ed25519.pub.toml") {
        for (dep, archive_path) in dep_pkgars.iter() {
            let Ok(mut package) = pkgar::PackageFile::new(archive_path, &key_file.pkey) else {
                continue;
            };
            let Ok(entries) = pkgar_core::PackageSrc::read_entries(&mut package) else {
                continue;
            };
            for entry in entries {
                let Ok(entry_path) = pkgar::ext::EntryExt::check_path(&entry) else {
                    continue;
                };
                for prefix in &["lib", "usr/lib"] {
                    let Ok(child_path) = entry_path.strip_prefix(prefix) else {
                        continue;
                    };
                    let Some(child_name) = child_path.to_str() else {
                        continue;
                    };
                    if needed.contains(child_name) {
                        eprintln!("DEBUG: {} provides {}", dep, child_name);
                        deps.insert(dep.clone());
                        missing.remove(child_name);
                    }
                }
            }
        }
    }

    for name in missing {
        eprintln!("WARN: {} missing", name);
    }

    deps
}

fn build(
    recipe_dir: &Path,
    source_dir: &Path,
    target_dir: &Path,
    name: &PackageName,
    recipe: &Recipe,
    offline_mode: bool,
    check_source: bool,
) -> Result<(PathBuf, BTreeSet<PackageName>), String> {
    let sysroot_dir = target_dir.join("sysroot");
    let stage_dir = target_dir.join("stage");

    let mut dep_pkgars = BTreeSet::new();
    for dependency in recipe.build.dependencies.iter() {
        let dependency_dir = recipes::find(dependency.as_str());
        if dependency_dir.is_none() {
            return Err(format!("failed to find recipe directory '{}'", dependency));
        }
        dep_pkgars.insert((
            dependency.clone(),
            dependency_dir
                .unwrap()
                .join("target")
                .join(redoxer::target())
                .join("stage.pkgar"),
        ));
    }

    if stage_dir.exists() && !check_source {
        let auto_deps = build_auto_deps(target_dir, &stage_dir, dep_pkgars)?;
        return Ok((stage_dir, auto_deps));
    }

    let source_modified = modified_dir_ignore_git(source_dir)?;
    let deps_modified = dep_pkgars
        .iter()
        .map(|(_dep, pkgar)| modified(pkgar))
        .max()
        .unwrap_or(Ok(SystemTime::UNIX_EPOCH))?;

    // Rebuild sysroot if source is newer
    //TODO: rebuild on recipe changes
    if sysroot_dir.is_dir() {
        let sysroot_modified = modified_dir(&sysroot_dir)?;
        if sysroot_modified < source_modified || sysroot_modified < deps_modified {
            eprintln!(
                "DEBUG: '{}' newer than '{}'",
                source_dir.display(),
                sysroot_dir.display()
            );
            remove_all(&sysroot_dir)?;
        }
    }
    if !sysroot_dir.is_dir() {
        // Create sysroot.tmp
        let sysroot_dir_tmp = target_dir.join("sysroot.tmp");
        create_dir_clean(&sysroot_dir_tmp)?;

        // Make sure sysroot/usr exists
        create_dir(&sysroot_dir_tmp.join("usr"))?;
        for folder in &["bin", "include", "lib", "share"] {
            // Make sure sysroot/usr/$folder exists
            create_dir(&sysroot_dir_tmp.join("usr").join(folder))?;

            // Link sysroot/$folder sysroot/usr/$folder
            symlink(Path::new("usr").join(folder), &sysroot_dir_tmp.join(folder))?;
        }

        for (_dep, archive_path) in &dep_pkgars {
            let public_path = "build/id_ed25519.pub.toml";
            pkgar::extract(
                public_path,
                &archive_path,
                sysroot_dir_tmp.to_str().unwrap(),
            )
            .map_err(|err| {
                format!(
                    "failed to install '{}' in '{}': {:?}",
                    archive_path.display(),
                    sysroot_dir_tmp.display(),
                    err
                )
            })?;
        }

        // Move sysroot.tmp to sysroot atomically
        rename(&sysroot_dir_tmp, &sysroot_dir)?;
    }

    // Rebuild stage if source is newer
    //TODO: rebuild on recipe changes
    if stage_dir.is_dir() {
        let stage_modified = modified_dir(&stage_dir)?;
        if stage_modified < source_modified || stage_modified < deps_modified {
            eprintln!(
                "DEBUG: '{}' newer than '{}'",
                source_dir.display(),
                stage_dir.display()
            );
            remove_all(&stage_dir)?;
        }
    }

    if !stage_dir.is_dir() {
        // Create stage.tmp
        let stage_dir_tmp = target_dir.join("stage.tmp");
        create_dir_clean(&stage_dir_tmp)?;

        // Create build, if it does not exist
        //TODO: flag for clean builds where build is wiped out
        let build_dir = target_dir.join("build");
        if !build_dir.is_dir() {
            create_dir_clean(&build_dir)?;
        }

        let pre_script = r#"# Common pre script
# Add cookbook bins to path
if [ -z "${IS_REDOX}" ]
then
export PATH="${COOKBOOK_ROOT}/bin:${PATH}"
fi

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
install_flags=
build_flags=--release
offline_flags=
if [ ! -z "${COOKBOOK_DEBUG}" ]
then
    install_flags=--debug
    build_flags=
    build_type=debug
    export CFLAGS="${CFLAGS} -g"
    export CPPFLAGS="${CPPFLAGS} -g"
fi

if [ ! -z "${COOKBOOK_OFFLINE}" ]
then
offline_flags=--offline
fi

# cargo template
COOKBOOK_CARGO="${COOKBOOK_REDOXER}"
function cookbook_cargo {
    "${COOKBOOK_CARGO}" install \
        --path "${COOKBOOK_SOURCE}/${PACKAGE_PATH}" \
        --root "${COOKBOOK_STAGE}/usr" \
        --locked \
        --no-track \
        ${install_flags} \
        ${offline_flags} \
         -j "${COOKBOOK_MAKE_JOBS}" "$@"
}

# helper for installing binaries that are cargo examples
function cookbook_cargo_examples {
    recipe="$(basename "${COOKBOOK_RECIPE}")"
    for example in "$@"
    do
        "${COOKBOOK_CARGO}" build \
            --manifest-path "${COOKBOOK_SOURCE}/${PACKAGE_PATH}/Cargo.toml" \
            --example "${example}" \
            ${build_flags} ${offline_flags} -j "${COOKBOOK_MAKE_JOBS}"
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
            --manifest-path "${COOKBOOK_SOURCE}/${PACKAGE_PATH}/Cargo.toml" \
            --package "${package}" \
            ${build_flags} ${offline_flags} -j "${COOKBOOK_MAKE_JOBS}"
        mkdir -pv "${COOKBOOK_STAGE}/usr/bin"
        cp -v \
            "target/${TARGET}/${build_type}/${package}" \
            "${COOKBOOK_STAGE}/usr/bin/${recipe}_${package}"
    done
}

# configure template
COOKBOOK_CONFIGURE="${COOKBOOK_SOURCE}/configure"
COOKBOOK_CONFIGURE_FLAGS=(
    --host="${GNU_TARGET}"
    --prefix="/usr"
    --disable-shared
    --enable-static
)
COOKBOOK_MAKE="make"

if [ -z "${COOKBOOK_MAKE_JOBS}" ]
then
if [ -z "${IS_REDOX}" ]
then
COOKBOOK_MAKE_JOBS="$(nproc)"
else
COOKBOOK_MAKE_JOBS="1"
fi
fi

function cookbook_configure {
    "${COOKBOOK_CONFIGURE}" "${COOKBOOK_CONFIGURE_FLAGS[@]}" "$@"
    "${COOKBOOK_MAKE}" -j "${COOKBOOK_MAKE_JOBS}"
    "${COOKBOOK_MAKE}" install DESTDIR="${COOKBOOK_STAGE}"
}

COOKBOOK_CMAKE="cmake"
COOKBOOK_NINJA="ninja"
COOKBOOK_CMAKE_FLAGS=(
    -DBUILD_SHARED_LIBS=False
    -DENABLE_SHARED=False
    -DENABLE_STATIC=True
)
function cookbook_cmake {
    cat > cross_file.cmake <<EOF
set(CMAKE_AR ${GNU_TARGET}-ar)
set(CMAKE_CXX_COMPILER ${GNU_TARGET}-g++)
set(CMAKE_C_COMPILER ${GNU_TARGET}-gcc)
set(CMAKE_FIND_ROOT_PATH ${COOKBOOK_SYSROOT})
set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
set(CMAKE_PLATFORM_USES_PATH_WHEN_NO_SONAME 1)
set(CMAKE_PREFIX_PATH, ${COOKBOOK_SYSROOT})
set(CMAKE_RANLIB ${GNU_TARGET}-ranlib)
set(CMAKE_SHARED_LIBRARY_SONAME_C_FLAG "-Wl,-soname,")
set(CMAKE_SYSTEM_NAME UnixPaths)
set(CMAKE_SYSTEM_PROCESSOR $(echo "${TARGET}" | cut -d - -f1))
EOF

    if [ -n "${CC_WRAPPER}" ]
    then
        echo "set(CMAKE_C_COMPILER_LAUNCHER ${CC_WRAPPER})" >> cross_file.cmake
        echo "set(CMAKE_CXX_COMPILER_LAUNCHER ${CC_WRAPPER})" >> cross_file.cmake
    fi

    "${COOKBOOK_CMAKE}" "${COOKBOOK_SOURCE}" \
        -DCMAKE_BUILD_TYPE=Release \
        -DCMAKE_CROSSCOMPILING=True \
        -DCMAKE_INSTALL_INCLUDEDIR=include \
        -DCMAKE_INSTALL_LIBDIR=lib \
        -DCMAKE_INSTALL_OLDINCLUDEDIR=/include \
        -DCMAKE_INSTALL_PREFIX=/usr \
        -DCMAKE_INSTALL_SBINDIR=bin \
        -DCMAKE_TOOLCHAIN_FILE=cross_file.cmake \
        -GNinja \
        -Wno-dev \
        "${COOKBOOK_CMAKE_FLAGS[@]}" \
        "$@"

    "${COOKBOOK_NINJA}" -j"${COOKBOOK_MAKE_JOBS}"
    DESTDIR="${COOKBOOK_STAGE}" "${COOKBOOK_NINJA}" install -j"${COOKBOOK_MAKE_JOBS}"
}

COOKBOOK_MESON="meson"
COOKBOOK_MESON_FLAGS=(
    --buildtype release
    --wrap-mode nofallback
    --strip
    -Ddefault_library=static
    -Dprefix=/usr
)
function cookbook_meson {
    echo "[binaries]" > cross_file.txt
    echo "c = [$(printf "'%s', " $CC | sed 's/, $//')]"  >> cross_file.txt
    echo "cpp = [$(printf "'%s', " $CXX | sed 's/, $//')]" >> cross_file.txt
    echo "ar = '${AR}'" >> cross_file.txt
    echo "strip = '${STRIP}'" >> cross_file.txt
    echo "pkg-config = '${PKG_CONFIG}'" >> cross_file.txt
    echo "llvm-config = '${TARGET}-llvm-config'" >> cross_file.txt
    echo "glib-compile-resources = 'glib-compile-resources'" >> cross_file.txt
    echo "glib-compile-schemas = 'glib-compile-schemas'" >> cross_file.txt

    echo "[host_machine]" >> cross_file.txt
    echo "system = 'redox'" >> cross_file.txt
    echo "cpu_family = '$(echo "${TARGET}" | cut -d - -f1)'" >> cross_file.txt
    echo "cpu = '$(echo "${TARGET}" | cut -d - -f1)'" >> cross_file.txt
    echo "endian = 'little'" >> cross_file.txt

    echo "[paths]" >> cross_file.txt
    echo "prefix = '/usr'" >> cross_file.txt
    echo "libdir = 'lib'" >> cross_file.txt
    echo "bindir = 'bin'" >> cross_file.txt

    echo "[properties]" >> cross_file.txt
    echo "needs_exe_wrapper = true" >> cross_file.txt
    echo "sys_root = '${COOKBOOK_SYSROOT}'" >> cross_file.txt
    echo "c_args = [$(printf "'%s', " $CFLAGS | sed 's/, $//')]" >> cross_file.txt
    echo "cpp_args = [$(printf "'%s', " $CPPFLAGS | sed 's/, $//')]" >> cross_file.txt
    echo "c_link_args = [$(printf "'%s', " $LDFLAGS | sed 's/, $//')]" >> cross_file.txt

    unset AR
    unset AS
    unset CC
    unset CXX
    unset LD
    unset NM
    unset OBJCOPY
    unset OBJDUMP
    unset PKG_CONFIG
    unset RANLIB
    unset READELF
    unset STRIP

    "${COOKBOOK_MESON}" setup \
        "${COOKBOOK_SOURCE}" \
        . \
        --cross-file cross_file.txt \
        "${COOKBOOK_MESON_FLAGS[@]}" \
        "$@"
    "${COOKBOOK_NINJA}" -j"${COOKBOOK_MAKE_JOBS}"
    DESTDIR="${COOKBOOK_STAGE}" "${COOKBOOK_NINJA}" install -j"${COOKBOOK_MAKE_JOBS}"
}

"#;

        let post_script = r#"# Common post script
# Strip binaries
for dir in "${COOKBOOK_STAGE}/bin" "${COOKBOOK_STAGE}/usr/bin"
do
    if [ -d "${dir}" ] && [ -z "${COOKBOOK_NOSTRIP}" ]
    then
        find "${dir}" -type f -exec "${GNU_TARGET}-strip" -v {} ';'
    fi
done

# Remove libtool files
for dir in "${COOKBOOK_STAGE}/lib" "${COOKBOOK_STAGE}/usr/lib"
do
    if [ -d "${dir}" ]
    then
        find "${dir}" -type f -name '*.la' -exec rm -fv {} ';'
    fi
done

# Remove cargo install files
for file in .crates.toml .crates2.json
do
    if [ -f "${COOKBOOK_STAGE}/${file}" ]
    then
        rm -v "${COOKBOOK_STAGE}/${file}"
    fi
done

# Add pkgname to appstream metadata
for dir in "${COOKBOOK_STAGE}/share/metainfo" "${COOKBOOK_STAGE}/usr/share/metainfo"
do
    if [ -d "${dir}" ]
    then
        find "${dir}" -type f -name '*.xml' -exec sed -i 's|</component>|<pkgname>'"${COOKBOOK_NAME}"'</pkgname></component>|g' {} ';'
    fi
done
"#;

        let flags_fn = |name, flags: &Vec<String>| {
            format!(
                "{name}+=(\n{}\n)\n",
                flags
                    .iter()
                    .map(|s| format!("  \"{s}\""))
                    .collect::<Vec<String>>()
                    .join("\n")
            )
        };

        //TODO: better integration with redoxer (library instead of binary)
        //TODO: configurable target
        //TODO: Add more configurability, convert scripts to Rust?
        let script = match &recipe.build.kind {
            BuildKind::Cargo {
                package_path,
                cargoflags,
            } => {
                format!(
                    "PACKAGE_PATH={} cookbook_cargo {cargoflags}",
                    package_path.as_deref().unwrap_or(".")
                )
            }
            BuildKind::Configure { configureflags } => format!(
                "DYNAMIC_INIT\n{}cookbook_configure",
                flags_fn("COOKBOOK_CONFIGURE_FLAGS", configureflags),
            ),
            BuildKind::Cmake { cmakeflags } => format!(
                "DYNAMIC_INIT\n{}cookbook_cmake",
                flags_fn("COOKBOOK_CMAKE_FLAGS", cmakeflags),
            ),
            BuildKind::Meson { mesonflags } => format!(
                "DYNAMIC_INIT\n{}cookbook_meson",
                flags_fn("COOKBOOK_MESON_FLAGS", mesonflags),
            ),
            BuildKind::Custom { script } => script.clone(),
            BuildKind::Remote => return build_remote(target_dir, name, offline_mode),
            BuildKind::None => "".to_owned(),
        };

        let command = {
            //TODO: remove unwraps
            let cookbook_build = build_dir.canonicalize().unwrap();
            let cookbook_recipe = recipe_dir.canonicalize().unwrap();
            let cookbook_root = Path::new(".").canonicalize().unwrap();
            let cookbook_stage = stage_dir_tmp.canonicalize().unwrap();
            let cookbook_source = source_dir.canonicalize().unwrap();
            let cookbook_sysroot = sysroot_dir.canonicalize().unwrap();

            let mut command = if is_redox() {
                let mut command = Command::new("bash");
                command.arg("-ex");
                command.env("COOKBOOK_REDOXER", "cargo");
                command
            } else {
                let cookbook_redoxer = Path::new("target/release/cookbook_redoxer")
                    .canonicalize()
                    .unwrap();
                let mut command = Command::new(&cookbook_redoxer);
                command.arg("env").arg("bash").arg("-ex");
                command.env("COOKBOOK_REDOXER", &cookbook_redoxer);
                command
            };
            command.current_dir(&cookbook_build);
            command.env("COOKBOOK_BUILD", &cookbook_build);
            command.env("COOKBOOK_NAME", name.as_str());
            command.env("COOKBOOK_RECIPE", &cookbook_recipe);
            command.env("COOKBOOK_ROOT", &cookbook_root);
            command.env("COOKBOOK_STAGE", &cookbook_stage);
            command.env("COOKBOOK_SOURCE", &cookbook_source);
            command.env("COOKBOOK_SYSROOT", &cookbook_sysroot);
            if offline_mode {
                command.env("COOKBOOK_OFFLINE", "1");
            }
            command
        };

        let full_script = format!(
            "{}\n{}\n{}\n{}",
            pre_script, SHARED_PRESCRIPT, script, post_script
        );
        run_command_stdin(command, full_script.as_bytes())?;

        // Move stage.tmp to stage atomically
        rename(&stage_dir_tmp, &stage_dir)?;
    }

    let auto_deps = build_auto_deps(target_dir, &stage_dir, dep_pkgars)?;

    Ok((stage_dir, auto_deps))
}

/// Calculate automatic dependencies
fn build_auto_deps(
    target_dir: &Path,
    stage_dir: &PathBuf,
    dep_pkgars: BTreeSet<(PackageName, PathBuf)>,
) -> Result<BTreeSet<PackageName>, String> {
    let auto_deps_path = target_dir.join("auto_deps.toml");
    if auto_deps_path.is_file() && modified(&auto_deps_path)? < modified(stage_dir)? {
        remove_all(&auto_deps_path)?
    }

    let auto_deps = if auto_deps_path.exists() {
        let toml_content =
            fs::read_to_string(&auto_deps_path).map_err(|_| "failed to read cached auto_deps")?;
        let wrapper: AutoDeps =
            toml::from_str(&toml_content).map_err(|_| "failed to deserialize cached auto_deps")?;
        wrapper.packages
    } else {
        let packages = auto_deps(stage_dir, &dep_pkgars);
        let wrapper = AutoDeps { packages };
        serialize_and_write(&auto_deps_path, &wrapper)?;
        wrapper.packages
    };
    Ok(auto_deps)
}

fn package(
    stage_dir: &Path,
    target_dir: &Path,
    name: &PackageName,
    recipe: &Recipe,
    auto_deps: &BTreeSet<PackageName>,
) -> Result<PathBuf, String> {
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
    // Rebuild package if stage is newer
    //TODO: rebuild on recipe changes
    if package_file.is_file() {
        let stage_modified = modified_dir(stage_dir)?;
        if modified(&package_file)? < stage_modified {
            eprintln!(
                "DEBUG: '{}' newer than '{}'",
                stage_dir.display(),
                package_file.display()
            );
            remove_all(&package_file)?;
        }
    }
    if !package_file.is_file() {
        pkgar::create(
            secret_path,
            package_file.to_str().unwrap(),
            stage_dir.to_str().unwrap(),
        )
        .map_err(|err| format!("failed to create pkgar archive: {:?}", err))?;

        package_toml(target_dir, name, recipe, auto_deps)?;
    }

    Ok(package_file)
}

fn package_toml(
    target_dir: &Path,
    name: &PackageName,
    recipe: &Recipe,
    auto_deps: &BTreeSet<PackageName>,
) -> Result<(), String> {
    let mut depends = recipe.package.dependencies.clone();
    for dep in auto_deps.iter() {
        if !depends.contains(dep) {
            depends.push(dep.clone());
        }
    }
    let package = Package {
        name: name.clone(),
        version: package_version(recipe),
        target: env::var("TARGET").map_err(|err| format!("failed to read TARGET: {:?}", err))?,
        depends,
    };

    serialize_and_write(&target_dir.join("stage.toml"), &package)?;

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

fn cook_meta(
    recipe_dir: &Path,
    name: &PackageName,
    recipe: &Recipe,
    fetch_only: bool,
) -> Result<(), String> {
    if fetch_only {
        return Ok(());
    }

    let target_dir = create_target_dir(recipe_dir)?;
    let empty_deps = BTreeSet::new();
    let _package_file = package_toml(&target_dir, name, recipe, &empty_deps)
        .map_err(|err| format!("failed to package: {}", err))?;

    Ok(())
}

fn cook(
    recipe_dir: &Path,
    name: &PackageName,
    recipe: &Recipe,
    is_deps: bool,
    fetch_only: bool,
    is_offline: bool,
) -> Result<(), String> {
    if recipe.build.kind == BuildKind::None {
        return cook_meta(recipe_dir, name, recipe, fetch_only);
    }

    let source_dir = match is_offline {
        true => fetch_offline(recipe_dir, &recipe.source),
        false => fetch(recipe_dir, &recipe.source),
    }
    .map_err(|err| format!("failed to fetch: {}", err))?;

    if fetch_only {
        return Ok(());
    }

    let target_dir = create_target_dir(recipe_dir)?;

    let (stage_dir, auto_deps) = build(
        recipe_dir,
        &source_dir,
        &target_dir,
        name,
        recipe,
        is_offline,
        !is_deps,
    )
    .map_err(|err| format!("failed to build: {}", err))?;

    let _package_file = package(&stage_dir, &target_dir, name, recipe, &auto_deps)
        .map_err(|err| format!("failed to package: {}", err))?;

    Ok(())
}

fn create_target_dir(recipe_dir: &Path) -> Result<PathBuf, String> {
    let target_parent_dir = recipe_dir.join("target");
    if !target_parent_dir.is_dir() {
        create_dir(&target_parent_dir)?;
    }
    let target_dir = target_parent_dir.join(redoxer::target());
    if !target_dir.is_dir() {
        create_dir(&target_dir)?;
    }
    Ok(target_dir)
}

fn main() {
    init_config();
    let mut matching = true;
    let mut dry_run = false;
    let mut fetch_only = false;
    let mut with_package_deps = false;
    let mut quiet = false;
    let mut nonstop = false;
    let mut is_offline = false;
    let mut recipe_names = Vec::new();
    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--" if matching => matching = false,
            "-d" | "--dry-run" if matching => dry_run = true,
            "--with-package-deps" if matching => with_package_deps = true,
            "--fetch-only" if matching => fetch_only = true,
            "-q" | "--quiet" if matching => quiet = true,
            "--nonstop" => nonstop = true,
            "--offline" => is_offline = true,
            _ => recipe_names.push(arg.try_into().expect("Invalid package name")),
        }
    }

    if with_package_deps {
        recipe_names = match CookRecipe::get_package_deps_recursive(&recipe_names, WALK_DEPTH) {
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
    }

    let recipes = match CookRecipe::get_build_deps_recursive(&recipe_names, !with_package_deps) {
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
        if !quiet {
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
            if !quiet {
                eprintln!("DRY RUN: {:#?}", recipe.recipe);
            }
            Ok(())
        } else {
            cook(
                &recipe.dir,
                &recipe.name,
                &recipe.recipe,
                recipe.is_deps,
                fetch_only,
                is_offline,
            )
        };

        match res {
            Ok(()) => {
                if !quiet {
                    eprintln!(
                        "{}{}cook - {} - successful{}{}",
                        style::Bold,
                        color::Fg(color::AnsiValue(46)),
                        recipe.name,
                        color::Fg(color::Reset),
                        style::Reset,
                    );
                }
            }
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
                if !nonstop {
                    process::exit(1);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix;

    use super::auto_deps;

    #[test]
    fn file_system_loop_no_infinite_loop() {
        // Hierarchy with an infinite loop
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let dir = root.join("loop");
        unix::fs::symlink(root, &dir).expect("Linking {dir:?} to {root:?}");

        // Sanity check that we have a loop
        assert_eq!(
            root.canonicalize().unwrap(),
            dir.canonicalize().unwrap(),
            "Expected a loop where {dir:?} points to {root:?}"
        );

        let entries = auto_deps(root, &Default::default());
        assert!(
            entries.is_empty(),
            "auto_deps shouldn't have yielded any libraries"
        );
    }
}
