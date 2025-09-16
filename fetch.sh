#!/usr/bin/env bash
set -e

source config.sh

cook --fetch-only ${@:1}
