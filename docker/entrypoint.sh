#!/bin/bash

# Add local user
# Either use the LOCAL_USER_ID if passed in at runtime or
# fallback

USER_ID=${LOCAL_USER_ID:-9001}

echo "Starting with UID : $USER_ID "
echo "CARGO_HOME is $CARGO_HOME"
echo "RUSTUP_HOME is $RUSTUP_HOME"
useradd --shell /bin/bash -u $USER_ID -o -c "" -m user
export HOME=/home/user
chown user:user -R $CARGO_HOME
chown user:user -R $RUSTUP_HOME

exec gosu user:user "$@"
