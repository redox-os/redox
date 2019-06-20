#!/usr/bin/env bash

# Add local user
# Either use LOCAL_UID and LOCAL_GID if passed in at runtime via
# -e LOCAL_UID="$(id -u)" -e LOCAL_GID="$(id -g)" or fallback
USER_NAME=redox
RUN_UID=${LOCAL_UID:-9001}
RUN_GID=${LOCAL_GID:-9001}

groupadd --non-unique --gid $RUN_GID $USER_NAME
useradd --non-unique --create-home --uid $RUN_UID --gid $USER_NAME --groups sudo $USER_NAME

echo "$USER_NAME ALL=(ALL) NOPASSWD: ALL" > /etc/sudoers.d/user-no-sudo-password

export HOME=/home/$USER_NAME

# Check current UID and GID of files in the named volume caches for
# cargo and rustup. Test only one of the top level folders to speed
# things up.
TESTFILE=$RUSTUP_HOME/settings.toml
CACHED_UID=$(stat -c "%u" $TESTFILE)
CACHED_GID=$(stat -c "%g" $TESTFILE)

if [ $CACHED_UID != $RUN_UID ] || [ $RUN_GID != $CACHED_GID ]; then
    echo -e "\033[01;38;5;155mChanging user id:group to ${RUN_UID}:${RUN_GID}. Please wait...\033[0m"
    chown $RUN_UID:$RUN_GID -R $CARGO_HOME $RUSTUP_HOME
fi

# fixes issue in docker for mac where fuse permissions are restricted to root:root
# https://github.com/theferrit32/data-commons-workspace/issues/5
# https://github.com/heliumdatacommons/data-commons-workspace/commit/f96624c8a55f5ded5ac60f4f54182a59be92e66d
if [ -f /dev/fuse ]; then chmod 666 /dev/fuse; fi

exec gosu $USER_NAME "$@"
