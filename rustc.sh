#!/bin/bash
have_o=false
for arg in "$@"; do
    if [[ "$arg" = "-o" ]]; then
        have_o=true
        break
    fi
done

args=()
for arg in "$@"; do
    if [[ $have_o = true && "$arg" =~ ^extra-filename= ]]; then
        unset args[${#args[@]}-1]
    elif [[ $have_o = true && "$arg" =~ ^--emit= ]]; then
        args+=("--emit=link")
    else
        args+=("$arg")
    fi
done

RUST_BACKTRACE=1 exec rustc -L build/userspace "${args[@]}"
