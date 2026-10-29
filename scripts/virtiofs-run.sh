#!/usr/bin/env bash
# Runs a command next to one virtiofsd per shared directory, for
# `make qemu share=...` and `make ch share=...`.
#   virtiofs-run.sh --share <socket>.sock <tag>=<dir>[:ro] [--share ...] -- <command...>

set -euo pipefail

STARTUP_TIMEOUT=5 # seconds

die() {
  echo "virtiofs: $*" >&2
  exit 1
}

usage() {
  echo "usage: $0 --share <socket>.sock <tag>=<dir>[:ro] [--share ...] -- <command...>" >&2
  exit 2
}

SOCKS=()
TAGS=()
DIRS=()
OPTS=()
LABELS=()
while [[ "${1:-}" == --share ]]; do
  [[ $# -ge 3 ]] || usage
  [[ "$3" == *=* ]] || die "share '$3': expected <tag>=<dir>"
  entry="$3" opt="" label=""
  if [[ "${entry}" == *:ro ]]; then
    entry="${entry%:ro}" opt=--readonly label=" (read-only)"
  fi
  OPTS+=("${opt}")
  LABELS+=("${label}")
  SOCKS+=("$2")
  TAGS+=("${entry%%=*}")
  DIRS+=("${entry#*=}")
  shift 3
done
[[ ${#SOCKS[@]} -gt 0 && "${1:-}" == -- && $# -ge 2 ]] || usage
shift # the "--"

# virtiofsd holds a lock on <socket>.pid for as long as it runs.
share_in_use() {
  [[ -e "$1.pid" ]] && ! flock -n "$1.pid" true
}

declare -A SEEN_TAGS=()
validate_share() {
  local sock="${SOCKS[$1]}" tag="${TAGS[$1]}" dir="${DIRS[$1]}"
  [[ -n "${tag}" ]] || die "share '=${dir}': empty tag"
  [[ "${tag}" =~ ^[A-Za-z0-9._-]+$ ]] \
    || die "tag '${tag}': only letters, digits, '.', '_' and '-' are allowed (is '${tag}=${dir}' a directory with an '=' in it? use tag=dir)"
  [[ ${#tag} -le 36 ]] || die "tag '${tag}': longer than 36 bytes"
  [[ -z "${SEEN_TAGS[${tag}]:-}" ]] || die "tag '${tag}' is used twice"
  SEEN_TAGS[${tag}]=1
  [[ -n "${dir}" ]] || die "share '${tag}': empty directory"
  [[ "${dir}" != *[[:space:]]* ]] \
    || die "share '${tag}': directories containing whitespace are not supported ('${dir}')"
  [[ -d "${dir}" ]] || die "share '${tag}': ${dir} is not a directory"
  [[ "${sock}" == *.sock ]] || die "socket '${sock}' must end in .sock"
  [[ ${#sock} -lt 108 ]] \
    || die "socket path too long (${#sock} bytes): ${sock}; use a shorter share_sockdir"
  if share_in_use "${sock}"; then
    die "${sock} is in use by another virtiofsd. Stop that VM first, or use another share_sockdir."
  fi
}

for i in "${!SOCKS[@]}"; do
  validate_share "${i}"
done

find_virtiofsd() {
  local c
  for c in "${VIRTIOFSD:-}" virtiofsd /usr/libexec/virtiofsd /usr/lib/qemu/virtiofsd \
    "${XDG_CACHE_HOME:-${HOME:-/tmp}/.cache}/virtiofsd-redox/virtiofsd"; do
    [[ -n "${c}" ]] && command -v "${c}" && return 0
  done
  return 1
}

VIRTIOFSD_BIN="$(find_virtiofsd)" \
  || die "virtiofsd not found. Install it (e.g. 'dnf install virtiofsd' or 'pacman -S virtiofsd'), run ./podman_bootstrap.sh -d, or set VIRTIOFSD=/path/to/virtiofsd"

PIDS=()

cleanup() {
  trap - EXIT INT TERM
  if [[ ${#PIDS[@]} -gt 0 ]]; then
    kill "${PIDS[@]}" 2>/dev/null || true
    wait "${PIDS[@]}" 2>/dev/null || true
  fi
  local sock
  for sock in "${SOCKS[@]}"; do
    rm -f "${sock}" "${sock}.pid"
  done
}

trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

declare -A SHARED_DIRS=()
start_virtiofsd() {
  local sock="${SOCKS[$1]}" dir
  mkdir -p "$(dirname "${sock}")"
  dir="$(cd "${DIRS[$1]}" && pwd)"
  if [[ -n "${SHARED_DIRS[${dir}]:-}" ]]; then
    echo "virtiofs: warning: ${dir} is shared more than once; the instances do not share caches" >&2
  fi
  SHARED_DIRS[${dir}]=1

  "${VIRTIOFSD_BIN}" \
    --socket-path="${sock}" \
    --shared-dir "${dir}" \
    --sandbox none \
    --log-level warn \
    --translate-uid="map:0:$(id -u):1" \
    --translate-gid="map:0:$(id -g):1" \
    ${OPTS[$1]} \
    </dev/null >"${sock%.sock}.log" 2>&1 &
  PIDS+=("$!")
}

start_failed() {
  local log="${SOCKS[$1]%.sock}.log"
  tail -n 20 "${log}" >&2 || true
  die "virtiofsd for '${TAGS[$1]}' $2; see ${log}"
}

wait_for_sockets() {
  local deadline=$((SECONDS + STARTUP_TIMEOUT)) i
  for i in "${!SOCKS[@]}"; do
    until [[ -S "${SOCKS[$i]}" ]]; do
      kill -0 "${PIDS[$i]}" 2>/dev/null || start_failed "${i}" "exited"
      ((SECONDS < deadline)) || start_failed "${i}" "did not create ${SOCKS[$i]} within ${STARTUP_TIMEOUT}s"
      sleep 0.1
    done
  done
}

for i in "${!SOCKS[@]}"; do
  start_virtiofsd "${i}"
done
wait_for_sockets
for i in "${!SOCKS[@]}"; do
  echo "virtiofs: sharing '${TAGS[$i]}' = ${DIRS[$i]}${LABELS[$i]} (log: ${SOCKS[$i]%.sock}.log)" >&2
done

# Not exec'd, so cleanup runs when the command ends.
"$@"
