#!/bin/sh
# Copyright 2015 The Rust Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution and at
# http://rust-lang.org/COPYRIGHT.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

# # Coding conventions
#
# * globals are `like_this`.
# * locals are `_like_this`.
# * exported values are `LIKE_THIS`.
# * out-of-band return values are put into `RETVAL`.
#
# # Error handling
#
# Oh, my goodness, error handling. It's terrifying.
#
# This doesn't use -e because it makes it hard to control the
# presentation of and response to errors.
#
# `set -u` is on, which means undefined variables are errors.
# Generally when evaluating a variable that may not exist I'll
# write `${mystery_variable-}`, which results in "" if the name
# is undefined.
#
# Every command should be expected to return 0 on success, and
# non-zero on failure. In one case, for `download_and_check`, the
# error code needs to be interpreted more carefully because there are
# multiple successful return codes. Additional return values may be
# passed the `$RETVAL` global or further `RETVAL_FOO` globals as
# needed.
#
# Most commands are executed via wrappers that provide extra diagnostics
# and error handling: `run`, which prints the command on failure, and
# returns the error code, `ignore` which does the same, but is used
# to indicate the error code won't be handled, and `ensure`, which
# prints the command on failure, and also exits the process.
#
# Pass errors on on: `run cmd arg1 arg2 || return 1`. `run` will run
# the command, printing it if it fails; the `|| return 1` passes the
# error on to the caller. `ensure cmd arg1 arg1`, runs the command,
# printing it if it fails, and termining execution.
#
# Don't make typos. You just have to be better than that.
#
# This code is very careful never to create empty paths. Any time a
# new string that will be used as a path is produced, it is checked
# with `assert_nz`. Likewise, pretty much any time a string is
# constructed via command invocation it needs to be tested against
# the empty string.
#
# Temporary files must be carefully deleted on every error path.

set -u # Undefined variables are errors

main() {
    assert_cmds
    set_globals
    handle_command_line_args "$@"
}

set_globals() {
    # Environment sanity checks
    assert_nz "$HOME" "\$HOME is undefined"
    assert_nz "$0" "\$0 is undefined"

    # Some constants
    version=0.0.1
    metadata_version=1

    # Find the location of the distribution server
    default_dist_server="https://static.rust-lang.org"
    insecure_dist_server="http://static-rust-lang-org.s3-website-us-west-1.amazonaws.com"
    dist_server="${RUSTUP_DIST_SERVER-$default_dist_server}"
    using_insecure_dist_server=false

    # Check to see if GNUPG version 2 is installed, falling back to using version 1 by default
    gpg_exe=gpg
    if command -v gpg2 > /dev/null 2>&1; then
        gpg_exe=gpg2
    fi

    # Disable https if we can gpg because cloudfront often gets our files out of sync
    if [ "$dist_server" = "$default_dist_server" ]; then
       if command -v "$gpg_exe" > /dev/null 2>&1; then
           dist_server="$insecure_dist_server"
           using_insecure_dist_server=true
       fi
    fi

    # The directory on the server containing the dist artifacts
    rust_dist_dir=dist

    default_channel="nightly"

    # Set up the rustup data dir
    rustup_dir="${RUSTUP_HOME-$HOME/.rustup}"
    assert_nz "$rustup_dir" "rustup_dir"

    # Install prefix can be set by the environment
    default_prefix="${RUSTUP_PREFIX-/usr/local}"
    default_save=false
    if [ -n "${RUSTUP_SAVE-}" ]; then
       default_save=true
    fi

    # Data locations
    version_file="$rustup_dir/rustup-version"
    temp_dir="$rustup_dir/tmp"
    dl_dir="$rustup_dir/dl"

    # Set up the GPG key
    official_rust_gpg_key="
-----BEGIN PGP PUBLIC KEY BLOCK-----
Version: GnuPG v1

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
Mve696B5tlHyc1KxjHR6w9GRsh4=
=5FXw
-----END PGP PUBLIC KEY BLOCK-----
"

    if [ -n "${RUSTUP_GPG_KEY-}" ]; then
       gpg_key=$(cat "$RUSTUP_GPG_KEY")
    else
       gpg_key="$official_rust_gpg_key"
    fi

    # This is just used by test.sh for testing sha256sum fallback to shasum
    sha256sum_cmd="${__RUSTUP_MOCK_SHA256SUM-sha256sum}"

    flag_verbose=false
    flag_yes=false

    if [ -n "${RUSTUP_VERBOSE-}" ]; then
       flag_verbose=true
    fi
}

# Ensuresthat ~/.rustup exists and uses the correct format
initialize_metadata() {
    local _disable_sudo="$1"

    verbose_say "checking metadata version"

    if [ "$rustup_dir" = "$HOME" ]; then
       err "RUSTUP_HOME is the same as HOME. this cannot be correct. aborting"
    fi

    # This tries to guard against dumb values of RUSTUP_HOME like ~/ since
    # rustup will delete the entire directory.
    if [ -e "$rustup_dir" -a ! -e "$version_file" ]; then
       say "rustup home dir exists at $rustup_dir but version file $version_file does not."
       say "this may be old rustup metadata, in which case it can be deleted."
       err "this is very suspicous. aborting."
    fi

    # Oh, my. We used to encourage people running this script as root,
    # and that resulted in users' ~/.rustup directories being owned by
    # root (running `sudo sh` doesn't change $HOME apparently). Now
    # that we're not running as root, we can't touch our ~/.rustup
    # directory. Try to fix that.
    if [ -e "$version_file" ]; then
       local _can_write=true
       local _probe_file="$rustup_dir/write-probe"
       ignore touch "$_probe_file" 2> /dev/null
       if [ $? != 0 ]; then
           _can_write=false
       else
           ensure rm "$_probe_file"
       fi

       if [ "$_can_write" = false ]; then
           say "$rustup_dir is unwritable. it was likely created by a previous rustup run under sudo"
           if [ "$_disable_sudo" = false ]; then
              say "deleting it with sudo"
              run sudo rm -R "$rustup_dir"
              if [ $? != 0 ]; then
                  err "unable to delete unwritable $rustup_dir"
              fi
           else
              say_err "not deleting it because of --disable-sudo"
              err "delete $rustup_dir to continue. aborting"
           fi
       fi
    fi

    ensure mkdir -p "$rustup_dir"
    rustup_dir="$(abs_path "$rustup_dir")"
    assert_nz "$rustup_dir" "rustup_dir"

    if [ ! -e "$version_file" ]; then
       verbose_say "writing metadata version $metadata_version"
       echo "$metadata_version" > "$version_file"
       need_ok "failed to write metadata version"
    else
       local _current_version="$(ensure cat "$version_file")"
       assert_nz "$_current_version"
       verbose_say "got metadata version $_current_version"
       if [ "$_current_version" != "$metadata_version" ]; then
           # Wipe the out of date metadata.
           say "metadata is out of date. deleting."
           ensure rm -R "$rustup_dir"
           ensure mkdir -p "$rustup_dir"
           echo "$metadata_version" > "$version_file"
           need_ok "failed to write metadata version"
       fi
    fi
}

handle_command_line_args() {
    local _save="$default_save"
    local _date=""
    local _prefix="$default_prefix"
    local _uninstall=false
    local _channel=""
    local _help=false
    local _revision=""
    local _spec=""
    local _update_hash_file=""
    local _disable_ldconfig=false
    local _disable_sudo=false

    for arg in "$@"; do
       case "$arg" in
           -s | --save )
              _save=true
              ;;
           -u | --uninstall )
              _uninstall=true
              ;;
           -h | --help )
              _help=true
              ;;

           -v | --verbose)
              # verbose is a global flag
              flag_verbose=true
              ;;

           --disable-ldconfig)
              _disable_ldconfig=true
              ;;

           --disable-sudo)
              _disable_sudo=true
              ;;

           -y | --yes)
              # yes is a global flag
              flag_yes=true
              ;;

           --version)
              echo "rustup.sh $version"
              exit 0
              ;;

       esac

       if is_value_arg "$arg" "prefix"; then
           _prefix="$(get_value_arg "$arg")"
       elif is_value_arg "$arg" "channel"; then
           _channel="$(get_value_arg "$arg")"
       elif is_value_arg "$arg" "date"; then
           _date="$(get_value_arg "$arg")"
       elif is_value_arg "$arg" "revision"; then
           _revision="$(get_value_arg "$arg")"
       elif is_value_arg "$arg" "spec"; then
           _spec="$(get_value_arg "$arg")"
       elif is_value_arg "$arg" "update-hash-file"; then
           # This option is used by multirust to short-circuit reinstalls
           # when the channel has not been updated by examining a content
           # hash in the update-hash-file
           _update_hash_file="$(get_value_arg "$arg")"
       fi
    done

    if [ "$_help" = true ]; then
       print_help
       exit 0
    fi

    # Make sure either rust256sum or shasum exists
    need_shasum_cmd

    # Check that the various toolchain-specifying flags don't conflict
    if [ -n "$_revision" ]; then
       if [ -n "$_channel" ]; then
           err "the --revision flag may not be combined with --channel"
       fi
       if [ -n "$_date" ]; then
           err "the --revision flag may not be combined with --date"
       fi
    fi

    if [ -n "$_spec" ]; then
       if [ -n "$_channel" ]; then
           err "the --spec flag may not be combined with --channel"
       fi
       if [ -n "$_revision" ]; then
           err "the --spec flag may not be combined with --revision"
       fi
    fi

    if [ -z "$_channel" -a -z "$_revision" -a -z "$_spec" ]; then
       _channel="$default_channel"
    fi

    # Toolchain can be either a channel, channel + date, or an explicit version
    local _toolchain=""
    if [ -n "$_channel" ]; then
       validate_channel "$_channel"
       _toolchain="$_channel"
       if [ -n "$_date" ]; then
           validate_date "$_date"
           _toolchain="$_toolchain-$_date"
       fi
    elif [ -n "$_revision" ]; then
       _toolchain="$_revision"
    elif [ -n "$_spec" ]; then
       _toolchain="$_spec"
    fi
    assert_nz "$_toolchain" "toolchain"

    if [ "$flag_yes" = false ]; then
       # Running in interactive mode, check that a tty exists
       check_tty

       # Print the welcome / warning message and wait for confirmation
       print_welcome_message "$_prefix" "$_uninstall" "$_disable_sudo"

       get_tty_confirmation
    fi

    # All work is done in the ~/.rustup dir, which will be deleted
    # afterward if the user doesn't pass --save. *If* ~/.rustup
    # already exists and they *did not* pass --save, we'll pretend
    # they did anyway to avoid deleting their data.
    local _preserve_rustup_dir="$_save"
    if [ "$_save" = false -a -e "$version_file" ]; then
       verbose_say "rustup home exists but not asked to save. saving anyway"
       _preserve_rustup_dir=true
    fi

    # Make sure our data directory exists and is the right format
    initialize_metadata "$_disable_sudo"

    # OK, time to do the things
    local _succeeded=true
    if [ "$_uninstall" = false ]; then
       install_toolchain_from_dist "$_toolchain" "$_prefix" "$_save" "$_update_hash_file" \
                                "$_disable_ldconfig" "$_disable_sudo"
       if [ $? != 0 ]; then
           _succeeded=false
       fi
    else
       remove_toolchain "$_prefix" "$_disable_sudo"
       if [ $? != 0 ]; then
           _succeeded=false
       fi
    fi

    # Remove the temporary directory.
    # This will not happen if we hit certain hard errors earlier.
    if [ "$_preserve_rustup_dir" = false ]; then
       verbose_say "removing rustup home $rustup_dir"
       ensure rm -R "$rustup_dir"
    else
       verbose_say "leaving rustup home $rustup_dir"
    fi

    if [ "$_succeeded" = false ]; then
       exit 1
    fi
}

is_value_arg() {
    local _arg="$1"
    local _name="$2"

    echo "$arg" | grep -q -- "--$_name="
    return $?
}

get_value_arg() {
    local _arg="$1"

    echo "$_arg" | cut -f2 -d=
}

validate_channel() {
    local _channel="$1"

    case "$_channel" in
       stable | beta | nightly )
           ;;
       * )
           err "channel must be either 'stable', 'beta', or 'nightly'"
           ;;
    esac
}

validate_date() {
    local _date="$1"

    case "$_date" in
       [0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9] )
           ;;
       * )
           err "date must be in YYYY-MM-DD format"
           ;;
    esac
}

print_welcome_message() {
    local _prefix="$1"
    local _uninstall="$2"
    local _disable_sudo="$3"

    cat <<EOF

Welcome to Rust.
EOF

    if [ "$_disable_sudo" = false ]; then
       if [ "$(id -u)" = 0 ]; then
           cat <<EOF

WARNING: This script appears to be running as root. While it will work
correctly, it is no longer necessary for rustup.sh to run as root.
EOF
       fi
    fi


    if [ "$_uninstall" = false ]; then
       cat <<EOF

This script will download the Rust compiler and its package manager, Cargo, and
install them to $_prefix. You may install elsewhere by running this script
with the --prefix=<path> option.
EOF
    else
       cat <<EOF

This script will uninstall the existing Rust installation at $_prefix.
EOF
    fi

    if [ "$_disable_sudo" = false ]; then
       cat <<EOF

The installer will run under 'sudo' and may ask you for your password. If you do
not want the script to run 'sudo' then pass it the --disable-sudo flag.
EOF
    fi

    if [ "$_uninstall" = false ]; then
       cat <<EOF

You may uninstall later by running $_prefix/lib/rustlib/uninstall.sh,
or by running this script again with the --uninstall flag.
EOF
    fi

    echo
}


# Updating toolchains

# Returns 0 on success, 1 on error
install_toolchain_from_dist() {
    local _toolchain="$1"
    local _prefix="$2"
    local _save="$3"
    local _update_hash_file="$4"
    local _disable_ldconfig="$5"
    local _disable_sudo="$6"

    # FIXME: Right now installing rust over top of multirust will
    # result in a broken multirust installation.
    # This hack tries to avoid that by detecting if multirust is installed,
    # but I'd rather fix this by having the installers understand negative
    # dependencies.
    local _potential_multirust_bin="$_prefix/bin/multirust"
    if [ -e "$_potential_multirust_bin" ]; then
       say_err "multirust appears to be installed at the destination, $_potential_multirust_bin"
       say_err "installing rust over multirust will result in breakage."
       local _potential_uninstaller="$_prefix/lib/rustlib/uninstall.sh"
       if [ -e "$_potential_uninstaller" ]; then
           say_err "consider uninstalling multirust first by running $_potential_uninstaller"
       fi
       err "aborting"
    fi

    if [ "$using_insecure_dist_server" = "true" ]; then
       # disabling https avoids rust#21293
       say "gpg available. signatures will be verified"
    else
       say "gpg not available. signatures will not be verified"
    fi

    determine_remote_rust_installer_location "$_toolchain" || return 1
    local _remote_rust_installer="$RETVAL"
    assert_nz "$_remote_rust_installer" "remote rust installer"
    verbose_say "remote rust installer location: $_remote_rust_installer"

    local _rust_installer_name="$(basename "$_remote_rust_installer")"
    assert_nz "$_rust_installer_name" "rust installer name"

    # Download and install toolchain
    say "downloading toolchain for '$_toolchain'"
    download_and_check "$_remote_rust_installer" false "$_update_hash_file"
    # Hey! I need to check $? twice here, so it has to be
    # assigned to a named variable, otherwise the second
    # check against $? will not be what I expect.
    local _retval=$?
    if [ "$_retval" = 20 ]; then
       say "'$_toolchain' is already up to date"
       # Successful short-circuit using the update-hash
       return 0
    fi
    if [ "$_retval" != 0 ]; then
       return 1
    fi
    local _installer_file="$RETVAL"
    local _installer_cache="$RETVAL_CACHE"
    local _update_hash="$RETVAL_UPDATE_HASH"
    assert_nz "$_installer_file" "installer_file"
    assert_nz "$_installer_cache" "installer_cache"
    assert_nz "$_update_hash" "update_hash"

    # Create a temp directory to put the downloaded toolchain
    make_temp_dir
    local _workdir="$RETVAL"
    assert_nz "$_workdir" "workdir"
    verbose_say "install work dir: $_workdir"

    # There next few statements may all fail independently.
    local _failing=false

    install_toolchain "$_toolchain" "$_installer_file" "$_workdir" "$_prefix" \
                    "$_disable_ldconfig" "$_disable_sudo"
    if [ $? != 0 ]; then
       say_err "failed to install toolchain"
       _failing=true
    fi

    run rm -R "$_workdir"
    if [ $? != 0 ]; then
       say_err "couldn't delete workdir"
       _failing=true
    fi

    # Throw away the cache if not --save
    if [ "$_save" = false ]; then
       verbose_say "discarding cache '$_installer_cache'"
       run rm -R "$_installer_cache"
       if [ $? != 0 ]; then
           say_err "couldn't delete cache dir"
           _failing=true
       fi
    fi

    # Write the update hash to file
    if [ -n "$_update_hash_file" ]; then
       echo "$_update_hash" > "$_update_hash_file"
       if [ $? != 0 ]; then
           say_err "failed to write update hash to file"
           _failing=true
       fi
    fi

    if [ "$_failing" = true ]; then
       return 1
    fi
}

install_toolchain() {
    local _toolchain="$1"
    local _installer="$2"
    local _workdir="$3"
    local _prefix="$4"
    local _disable_ldconfig="$5"
    local _disable_sudo="$6"

    local _installer_dir="$_workdir/$(basename "$_installer" | sed s/.tar.gz$//)"

    # Extract the toolchain
    say "extracting installer"
    run tar xzf "$_installer" -C "$_workdir"
    if [ $? != 0 ]; then
       verbose_say "failed to extract installer"
       return 1
    fi

    # Install the toolchain
    local _toolchain_dir="$_prefix"
    verbose_say "installing toolchain to '$_toolchain_dir'"
    say "installing toolchain for '$_toolchain'"

    if [ "$_disable_ldconfig" = false ]; then
       maybe_sudo "$_disable_sudo" sh "$_installer_dir/install.sh" --prefix="$_toolchain_dir"
    else
       maybe_sudo "$_disable_sudo" sh "$_installer_dir/install.sh" --prefix="$_toolchain_dir" --disable-ldconfig
    fi
    if [ $? != 0 ]; then
       verbose_say "failed to install toolchain"
       return 1
    fi

}

remove_toolchain() {
    local _prefix="$1"
    local _disable_sudo="$2"
    local _uninstall_script="$_prefix/lib/rustlib/uninstall.sh"

    if [ -e "$_uninstall_script" ]; then
       verbose_say "uninstalling from '$_uninstall_script'"
       maybe_sudo "$_disable_sudo" sh "$_uninstall_script"
       if [ $? != 0 ]; then
           say_err "failed to remove toolchain"
           return 1;
       fi
       say "toolchain '$_toolchain' uninstalled"
    else
       say "no toolchain installed at '$_prefix'"
    fi
}

# Manifest interface

determine_remote_rust_installer_location() {
    local _toolchain="$1"

    verbose_say "determining remote rust installer for '$_toolchain'"

    case "$_toolchain" in
       nightly | beta | stable | nightly-* | beta-* | stable-* )
           download_rust_manifest "$_toolchain" || return 1
           local _manifest_file="$RETVAL"
           assert_nz "$_manifest_file" "manifest file"
           local _manifest_cache="$RETVAL_CACHE"
           assert_nz "$_manifest_cache" "manifest cache"
           get_remote_installer_location_from_manifest "$_toolchain" "$_manifest_file" rust "$rust_dist_dir" || return 1
           RETVAL="$RETVAL"
           verbose_say "deleting cache dir $_manifest_cache"
           run rm -R "$_manifest_cache" || return 1
           ;;

       * )
           verbose_say "interpreting toolchain spec as explicit version"
           get_architecture || return 1
           local _arch="$RETVAL"
           assert_nz "$_arch" "arch"

           local _file_name="rust-$_toolchain-$_arch.tar.gz"
           RETVAL="$dist_server/$rust_dist_dir/$_file_name"
           ;;
    esac
}

# Returns 0 on success.
# Returns the manifest file in RETVAL and it's cache dir in RETVAL_CACHE.
download_rust_manifest() {
    local _toolchain="$1"

    case "$_toolchain" in
       nightly | beta | stable )
           local _remote_rust_manifest="$dist_server/$rust_dist_dir/channel-rust-$_toolchain"
           ;;

       nightly-* | beta-* | stable-* )
           extract_channel_and_date_from_toolchain "$_toolchain" || return 1
           local _channel="$RETVAL_CHANNEL"
           local _date="$RETVAL_DATE"
           assert_nz "$_channel" "channel"
           assert_nz "$_date" "date"
           local _remote_rust_manifest="$dist_server/$rust_dist_dir/$_date/channel-rust-$_channel"
           ;;

       *)
           err "unrecognized toolchain spec: $_toolchain"
           ;;

    esac

    download_manifest "$_toolchain" "rust" "$_remote_rust_manifest" || return 1
    RETVAL="$RETVAL"
    RETVAL_CACHE="$RETVAL_CACHE"
}

download_manifest()  {
    local _toolchain="$1"
    local _name="$2"
    local _remote_manifest="$3"

    verbose_say "remote $_name manifest: $_remote_manifest"

    say "downloading manifest for '$_toolchain'"
    # It's not possible for $? = 20 here, because the update_hash_file
    # param is empty
    download_and_check "$_remote_manifest" true "" || return 1
    RETVAL="$RETVAL"
    RETVAL_CACHE="$RETVAL_CACHE"
}

get_remote_installer_location_from_manifest() {
    local _toolchain="$1"
    local _manifest_file="$2"
    local _package_name="$3"
    local _dist_dir="$4"

    if [ ! -e "$_manifest_file" ]; then
       err "manifest file '$_manifest_file' does not exist"
    fi

    get_architecture
    local _arch="$RETVAL"
    assert_nz "$_arch" "arch"

    while read _line; do
       # This regex checks for the version in addition to the package name because there
       # are package names that are substrings of other packages, 'rust-docs' vs. 'rust'.
       echo "$_line" | egrep "^$_package_name-(nightly|beta|alpha|[0-9]).*$_arch\.tar\.gz" > /dev/null
       if [ $? = 0 ]; then
           case "$_toolchain" in
              nightly | beta | stable )
                  RETVAL="$dist_server/$_dist_dir/$_line"
                  ;;

              nightly-* | beta-* | stable-* )
                  extract_channel_and_date_from_toolchain "$_toolchain" || return 1
                  local _channel="$RETVAL_CHANNEL"
                  local _date="$RETVAL_DATE"
                  assert_nz "$_channel" "channel"
                  assert_nz "$_date" "date"
                  RETVAL="$dist_server/$_dist_dir/$_date/$_line"
                  ;;

              *)
                  err "unrecognized toolchain spec: $_toolchain"
                  ;;
           esac
           return
       fi
    done < "$_manifest_file"

    err "couldn't find remote installer for '$_arch' in manifest"
}

extract_channel_and_date_from_toolchain() {
    local _toolchain="$1"

    case "$_toolchain" in
       nightly-20[0-9][0-9]-[0-9][0-9]-[0-9][0-9] | \
       beta-20[0-9][0-9]-[0-9][0-9]-[0-9][0-9] | \
       stable-20[0-9][0-9]-[0-9][0-9]-[0-9][0-9] )
           local _channel="$(ensure echo "$_toolchain" | ensure cut -d- -f1)"
           assert_nz "$_channel" "channel"
           local _date="$(ensure echo "$_toolchain" | ensure cut -d- -f2,3,4)"
           assert_nz "$_date" "date"
           RETVAL_CHANNEL="$_channel"
           RETVAL_DATE="$_date"
           ;;

       *)
           err "unrecognized toolchain spec: $_toolchain"
           ;;

    esac
}

# Tools

# FIXME: Temp names based on pid need to worry about pid recycling
make_temp_name() {
    local _pid="$$"
    assert_nz "$_pid" "pid"

    local _tmp_number="${NEXT_TMP_NUMBER-0}"
    local _tmp_name="tmp-$_pid-$_tmp_number"
    NEXT_TMP_NUMBER=$((_tmp_number + 1))
    need_ok "failed to create temp number"
    assert_nz "$NEXT_TMP_NUMBER" "NEXT_TMP_NUMBER"
    RETVAL="$_tmp_name"
}

make_temp_dir() {
    ensure mkdir -p "$temp_dir"

    ensure make_temp_name
    local _tmp_name="$temp_dir/$RETVAL"
    ensure mkdir -p "$_tmp_name"
    RETVAL="$_tmp_name"
}

# Returns 0 on success, like sha256sum
check_sums() {
    local _sumfile="$1"

    # Hackily edit the sha256 file to workaround a bug in the bots' generation of sums
    make_temp_dir
    local _workdir="$RETVAL"
    assert_nz "$_workdir" "workdir"

    sed s/tmp\\/dist\\/.*\\/final\\/// "$_sumfile" > "$_workdir/tmpsums"
    need_ok "failed to generate temporary checksums"

    local _sumfile_dirname="$(dirname "$_sumfile")"
    assert_nz "$_sumfile_dirname" "sumfile_dirname"
    if command -v "$sha256sum_cmd" > /dev/null 2>&1; then
       (run cd "$_sumfile_dirname" && run "$sha256sum_cmd" -c "$_workdir/tmpsums" > /dev/null)
    elif command -v shasum > /dev/null 2>&1; then
       (run cd "$_sumfile_dirname" && run shasum -c -a 256 "$_workdir/tmpsums" > /dev/null)
    else
       err "need either sha256sum or shasum"
    fi
    local _sum_retval=$?

    run rm -R "$_workdir" || return 1

    return $_sum_retval
}

# Outputs 40-char sum to stdout
create_sum() {
    local _input="$1"

    local _sum="none"
    if command -v "$sha256sum_cmd" > /dev/null 2>&1; then
       _sum="$(run "$sha256sum_cmd" "$_input" | run head -c 40)"
    elif command -v shasum > /dev/null 2>&1; then
       _sum="$(run shasum -a 256 "$_input" | run head -c 40)"
    else
       err "need either sha256sum or shasum"
    fi
    local _sum_retval=$?
    assert_nz "$_sum" "sum"

    ensure printf "$_sum"
    return  $_sum_retval
}

need_shasum_cmd() {
    if ! command -v "$sha256sum_cmd" > /dev/null 2>&1; then
       if ! command -v shasum > /dev/null 2>&1; then
           err "need either sha256sum or shasum"
       else
           verbose_say "sha256sum not available. falling back to shasum"
       fi
    fi
}

get_architecture() {

    verbose_say "detecting architecture"

    local _ostype="$(uname -s)"
    local _cputype="$(uname -m)"

    verbose_say "uname -s reports: $_ostype"
    verbose_say "uname -m reports: $_cputype"

    if [ "$_ostype" = Darwin -a "$_cputype" = i386 ]; then
       # Darwin `uname -s` lies
       if sysctl hw.optional.x86_64 | grep -q ': 1'; then
           local _cputype=x86_64
       fi
    fi

    case "$_ostype" in

       Linux)
           local _ostype=unknown-linux-gnu
           ;;

       FreeBSD)
           local _ostype=unknown-freebsd
           ;;

       DragonFly)
           local _ostype=unknown-dragonfly
           ;;

       Darwin)
           local _ostype=apple-darwin
           ;;

       MINGW* | MSYS*)
           err "unimplemented windows arch detection"
           ;;

       *)
           err "unrecognized OS type: $_ostype"
           ;;

    esac

    case "$_cputype" in

       i386 | i486 | i686 | i786 | x86)
            local _cputype=i686
            ;;

       xscale | arm)
           local _cputype=arm
            ;;

       armv7l)
            local _cputype=arm
            local _ostype="${_ostype}eabihf"
            ;;

       x86_64 | x86-64 | x64 | amd64)
            local _cputype=x86_64
            ;;

       *)
            err "unknown CPU type: $CFG_CPUTYPE"

    esac

    # Detect 64-bit linux with 32-bit userland
    if [ $_ostype = unknown-linux-gnu -a $_cputype = x86_64 ]; then
       # $SHELL does not exist in standard 'sh', so probably only exists
       # if configure is running in an interactive bash shell. /usr/bin/env
       # exists *everywhere*.
       local _bin_to_probe="$SHELL"
       if [ ! -e "$_bin_to_probe" -a -e "/usr/bin/env" ]; then
           _bin_to_probe="/usr/bin/env"
       fi
       if [ -e "$_bin_to_probe" ]; then
           file -L "$_bin_to_probe" | grep -q "x86[_-]64"
           if [ $? != 0 ]; then
              local _cputype=i686
           fi
       fi
    fi

    local _arch="$_cputype-$_ostype"
    verbose_say "architecture is $_arch"

    RETVAL="$_arch"
}

check_sig() {
    local _sig_file="$1"
    local _quiet="$2"

    if ! command -v "$gpg_exe" > /dev/null 2>&1; then
       return
    fi

    make_temp_dir
    local _workdir="$RETVAL"
    assert_nz "$_workdir" "workdir"
    verbose_say "sig work dir: $_workdir"

    echo "$gpg_key" > "$_workdir/key.asc"
    need_ok "failed to serialize gpg key"

    # Convert the armored key to .gpg format so it works with --keyring
    verbose_say "converting armored key to gpg"
    run "$gpg_exe" --no-permission-warning --dearmor "$_workdir/key.asc"
    if [ $? != 0 ]; then
       ignore rm -R "$_workdir"
       return 1
    fi

    verbose_say "verifying signature '$_sig_file'"
    local _output="$("$gpg_exe" --no-permission-warning --keyring "$_workdir/key.asc.gpg" --verify "$_sig_file" 2>&1)"
    if [ $? != 0 ]; then
       ignore echo "$_output"
       say_err "signature verification failed"
       ignore rm -R "$_workdir"
       return 1
    fi

    if [ "$_quiet" = false -o "$flag_verbose" = true ]; then
       ensure echo "$_output"
    fi

    run rm -R "$_workdir" || return 1
}

# Downloads a remote file, its checksum, and signature and verifies them.
# Returns 0 on success. Returns the path to the downloaded file in RETVAL,
# and the path to it's directory in the cache in RETVAL_CACHE.
#
# The caller can decide to remove it from the cache by deleting RETVAL_CACHE.
#
# A return code of *20* indicates a successful short circuit from the
# update hash.
download_and_check() {
    local _remote_name="$1"
    local _quiet="$2"
    local _update_hash_file="$3"

    local _remote_basename="$(basename "$_remote_name")"

    make_temp_dir
    local _workdir="$RETVAL"
    assert_nz "$_workdir" "workdir"
    verbose_say "download work dir: $_workdir"

    download_checksum_for "$_remote_name" "$_workdir/$_remote_basename"
    if [ $? != 0 ]; then
       ignore rm -R "$_workdir"
       return 1
    fi

    # This is the unique name of the cache, based on the content hash
    local _cache_name="$(create_sum "$_workdir/$_remote_basename.sha256" | head -c 20)"
    need_ok "failed to name cache name from checksum"
    assert_nz "$_cache_name" "cache_name"

    # If the user already has this rev then don't redownload it
    if [ -n "$_update_hash_file" ]; then
       # NB: May fail if file does not exist
       local _update_hash="$(cat "$_update_hash_file" 2> /dev/null)"

       verbose_say "provided update hash: $_update_hash"
       verbose_say "new update hash: $_cache_name"

       if [ "$_cache_name" = "$_update_hash" ]; then
           run rm -R "$_workdir" || return 1
           # NB: Return code 20 is successful here!
           return 20
       fi
    fi

    # Create a cache directory under dl_dir for this download, based off the content hash
    local _cache_dir="$dl_dir/$_cache_name"
    verbose_say "cache dir: $_cache_dir"
    run mkdir -p "$_cache_dir"
    if [ $? != 0 ]; then
       say_err "failed to create download directory"
       ignore rm -R "$_workdir"
       return 1
    fi

    # Move the checksum into the cache. -f because the file may
    # already exist from previous download.
    verbose_say "moving '$_workdir/$_remote_basename.sha256' to '$_cache_dir/$_remote_basename.sha256'"
    run mv -f "$_workdir/$_remote_basename.sha256" "$_cache_dir/$_remote_basename.sha256"
    if [ $? != 0 ]; then
       say_err "failed to move checksum into download cache"
       ignore rm -R "$_workdir"
       ignore rm -R "$_cache_dir"
       return 1
    fi

    # Done with the workdir
    run rm -R "$_workdir"
    if [ $? != 0 ]; then
       say_err "couldn't delete workdir '$_workdir'"
       ignore rm -R "$_cache_dir"
       return 1
    fi

    download_file_and_sig "$_remote_name" "$_cache_dir" "$_quiet"
    if [ $? != 0 ]; then
       # Leave the cache dir to resume the download later
       return 1
    fi
    check_file_and_sig "$_cache_dir/$_remote_basename" "$_quiet"
    if [ $? != 0 ]; then
       # Whatever's in the cache doesn't add up. Delete it.
       ignore rm -R "$_cache_dir"
       return 1
    fi

    RETVAL="$_cache_dir/$_remote_basename"
    RETVAL_CACHE="$_cache_dir"
    RETVAL_UPDATE_HASH="$_cache_name"
}

download_checksum_for() {
    local _remote_name="$1"
    local _local_name="$2"

    local _remote_sums="$_remote_name.sha256"
    local _local_sums="$_local_name.sha256"

    local _remote_basename="$(basename "$_remote_name")"
    local _remote_sums_basename="$_remote_basename.sha256"
    assert_nz "$_remote_basename" "remote basename"

    make_temp_dir
    local _workdir="$RETVAL"
    assert_nz "$_workdir" "workdir"
    verbose_say "download work dir: $_workdir"

    verbose_say "downloading '$_remote_sums' to '$_workdir'"
    (run cd "$_workdir" && run curl -s -f -O "$_remote_sums")
    if [ $? != 0 ]; then
       say_err "couldn't download checksum file '$_remote_sums'"
       ignore rm -R "$_workdir"
       return 1
    fi

    verbose_say "moving '$_workdir/$_remote_sums_basename' to '$_local_sums'"
    run mv -f "$_workdir/$_remote_sums_basename" "$_local_sums"
    if [ $? != 0 ]; then
       say_err "couldn't move '$_workdir/$_remote_sums_basename' to '$_local_sums'"
       ignore rm -R "$_workdir"
       return 1
    fi

    run rm -R "$_workdir"
    if [ $? != 0 ]; then
       say_err "couldn't delete workdir '$_workdir'"
       return 1
    fi
}

download_file_and_sig() {
    local _remote_name="$1"
    local _local_dirname="$2"
    local _quiet="$3"

    local _remote_basename="$(basename "$_remote_name")"
    assert_nz "$_remote_basename" "remote basename"

    local _local_name="$_local_dirname/$_remote_basename"

    local _remote_sig="$_remote_name.asc"
    local _local_sig="$_local_name.asc"

    # curl -C does not seem to work when the file already exists at 100%,
    # so just delete it and redownload.
    if [ -e "$_local_sig" ]; then
       run rm "$_local_sig"
       if [ $? != 0 ]; then
           say_err "failed to delete existing local signature for '$_remote_name'"
           return 1
       fi
    fi

    verbose_say "downloading '$_remote_sig' to '$_local_sig'"
    (run cd "$_local_dirname" && run curl -s -C - -f -O "$_remote_sig")
    if [ $? != 0 ]; then
       say_err "couldn't download signature file '$_remote_sig'"
       return 1
    fi

    # Again, because curl -C doesn't like a complete file, short circuit
    # curl by checking the sum.
    local _local_sums_file="$_local_dirname/$_remote_basename.sha256"
    # Throwing away error text since this error is expected.
    check_sums "$_local_sums_file" > /dev/null 2>&1
    if [ $? = 0 ]; then
       return 0
    fi

    verbose_say "downloading '$_remote_name' to '$_local_name'"
    # Invoke curl in a way that will resume if necessary
    if [ "$_quiet" = false ]; then
       (run cd "$_local_dirname" && run curl -# -C - -f -O "$_remote_name")
    else
       (run cd "$_local_dirname" && run curl -s -C - -f -O "$_remote_name")
    fi
    if [ $? != 0 ]; then
       say_err "couldn't download '$_remote_name'"
       return 1
    fi
}

check_file_and_sig() {
    local _local_name="$1"
    local _quiet="$2"

    local _local_sums="$_local_name.sha256"
    local _local_sig="$_local_name.asc"

    verbose_say "verifying checksums for '$_local_name'"
    check_sums "$_local_sums"
    if [ $? != 0 ]; then
       say_err "checksum failed for '$_local_name'"
       return 1
    fi

    check_sig "$_local_sig" "$_quiet"
    if [ $? != 0 ]; then
       say_err "signature failed for '$_local_name'"
       return 1
    fi
}

# Verifies that the terminal can be opened or exits
check_tty() {
    if [ ! -r /dev/tty ]; then
       err "running in interactive mode (without -y), but cannot open /dev/tty"
    fi
}

# Waits for a y/n response and exits if n
get_tty_confirmation() {
    local _yn=""
    read -p "Continue? (y/N) " _yn < /dev/tty
    need_ok "failed to read from /dev/tty"

    echo

    if [ "$_yn" != "y" -a "$_yn" != "Y" -a "$_yn" != "yes" ]; then
       say "cancelling"
       exit 0
    fi
}

maybe_sudo() {
    local _disable_sudo="$1"

    shift

    if [ "$_disable_sudo" = false ]; then
       run sudo "$@"
    else
       run "$@"
    fi
}

# Help

print_help() {
echo '
Usage: rustup.sh [--verbose|-v]

Options:

     --channel=(stable|beta|nightly)   Install from channel (default nightly)
     --date=<YYYY-MM-DD>               Install from archives
     --revision=<version-number>       Install a specific release
     --spec=<toolchain-spec>           Install from toolchain spec
     --prefix=<path>                   Install to a specific location (default /usr/local)
     --uninstall, u                    Uninstall instead of install
     --disable-ldconfig                Do not run ldconfig on Linux
     --disable-sudo                    Do not run installer under sudo
     --save, -s                        Save downloads for future reuse
     --yes, -y                         Disable the interactive mode
'
}

# Standard utilities

say() {
    echo "rustup: $1"
}

say_err() {
    say "$1" >&2
}

verbose_say() {
    if [ "$flag_verbose" = true ]; then
       say "$1"
    fi
}

err() {
    say "$1" >&2
    exit 1
}

need_cmd() {
    if ! command -v "$1" > /dev/null 2>&1
    then err "need $1"
    fi
}

need_ok() {
    if [ $? != 0 ]; then err "$1"; fi
}

assert_nz() {
    if [ -z "$1" ]; then err "assert_nz $2"; fi
}

# Run a command that should never fail. If the command fails execution
# will immediately terminate with an error showing the failing
# command.
ensure() {
    "$@"
    need_ok "command failed: $*"
}

# This is just for indicating that commands' results are being
# intentionally ignored. Usually, because it's being executed
# as part of error handling.
ignore() {
    run "$@"
}

# Runs a command and prints it to stderr if it fails.
run() {
    "$@"
    local _retval=$?
    if [ $_retval != 0 ]; then
       say_err "command failed: $*"
    fi
    return $_retval
}

# Prints the absolute path of a directory to stdout
abs_path() {
    local _path="$1"
    # Unset CDPATH because it causes havok: it makes the destination unpredictable
    # and triggers 'cd' to print the path to stdout. Route `cd`'s output to /dev/null
    # for good measure.
    (unset CDPATH && cd "$_path" > /dev/null && pwd)
}

assert_cmds() {
    need_cmd dirname
    need_cmd basename
    need_cmd mkdir
    need_cmd cat
    need_cmd curl
    need_cmd mktemp
    need_cmd rm
    need_cmd egrep
    need_cmd grep
    need_cmd file
    need_cmd uname
    need_cmd tar
    need_cmd sed
    need_cmd sh
    need_cmd mv
    need_cmd awk
    need_cmd cut
    need_cmd sort
    need_cmd date
    need_cmd head
    need_cmd printf
    need_cmd touch
    need_cmd id
}

main "$@"

# vim: set noet ts=8 sts=4 sw=4:
