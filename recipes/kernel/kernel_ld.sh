#!/usr/bin/env bash

set -ex

LD="$1"
shift

if "${LD}" -z use-gs-for-tls 2>&1 |
grep "warning: -z use-gs-for-tls ignored" &> /dev/null
then
	echo "Please update your prefix:" >&2
	echo "  rm -rf prefix" >&2
	echo "  make prefix" >&2
	exit 1
fi

exec "${LD}" -z use-gs-for-tls "$@"
