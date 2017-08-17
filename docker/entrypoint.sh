#!/usr/bin/env bash

# Use -e LOCAL_UID="$(id -u)" -e LOCAL_GID="$(id -g)"
# on the docker run command line if the container build user is different
# from the run user

CONT_UID=`id -u user`
CONT_GID=`id -g user`
RUN_UID=${LOCAL_UID:-$CONT_UID}
RUN_GID=${LOCAL_GID:-$CONT_GID}

if [ $RUN_UID != $CONT_UID ] || [ $RUN_GID != $CONT_GID ]; then
    echo -e "\033[01;38;5;155mChanging user id:group to ${RUN_UID}:${RUN_GID}. Please wait...\033[0m"
    groupmod -g $RUN_GID user
    usermod -u $RUN_UID -g $RUN_GID user
fi

exec gosu user:user "$@"
