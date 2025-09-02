#!/usr/bin/env bash
set -e

source config.sh

target/release/cook --fetch-only ${@:1}
