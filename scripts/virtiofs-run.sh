#!/usr/bin/env bash
# Run a command (normally QEMU) next to one virtiofsd per shared host directory.
#
#   virtiofs-run.sh --share <socket> <tag>=<dir> [--share ...] -- <command...>
#
# Used by `make qemu share=...` (see mk/qemu.mk). Each virtiofsd is started in
# the background and logs to <socket minus .sock>.log. All of them are stopped
# again when the command exits, fails to start, or is interrupted.
#
# virtiofsd is taken from $VIRTIOFSD, PATH, /usr/libexec, /usr/lib/qemu or
# ~/.cache/virtiofsd-redox. podman_bootstrap.sh installs it.

set -euo pipefail

die() {
  echo "virtiofs: $*" >&2
  exit 1
}

usage() {
  echo "usage: $0 --share <socket> <tag>=<dir> [--share ...] -- <command...>" >&2
  exit 2
}

SOCKS=()
SPECS=()
while [[ "${1:-}" == --share ]]; do
  [[ $# -ge 3 ]] || usage
  SOCKS+=("$2")
  SPECS+=("$3")
  shift 3
done
[[ ${#SOCKS[@]} -gt 0 && "${1:-}" == -- && $# -gt 1 ]] || usage
shift

# Validate everything before starting anything, so a mistake leaves nothing running
seen_tags=" "
for i in "${!SOCKS[@]}"; do
  sock="${SOCKS[i]}" spec="${SPECS[i]}"
  [[ "${spec}" == *=* ]] || die "share '${spec}': expected <tag>=<dir>"
  tag="${spec%%=*}" dir="${spec#*=}"
  [[ "${tag}" =~ ^[A-Za-z0-9._-]+$ ]] \
    || die "tag '${tag}': only letters, digits, '.', '_' and '-' are allowed (is '${spec}' a directory with an '=' in it? use tag=dir)"
  [[ ${#tag} -le 36 ]] || die "tag '${tag}': longer than 36 bytes"
  [[ "${seen_tags}" != *" ${tag} "* ]] || die "tag '${tag}' is used twice"
  seen_tags+="${tag} "
  [[ -n "${dir}" ]] || die "share '${tag}': empty directory"
  [[ "${dir}" != *[[:space:]]* ]] \
    || die "share '${tag}': directories containing whitespace are not supported ('${dir}')"
  [[ "${sock}" == *.sock ]] || die "socket '${sock}' must end in .sock"
  # AF_UNIX sun_path is 108 bytes including the NUL
  [[ ${#sock} -lt 108 ]] \
    || die "socket path too long (${#sock} bytes): ${sock}; use a shorter share_sockdir"
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
  rm -f "${SOCKS[@]}" "${SOCKS[@]/%/.pid}"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

resolved_dirs=" "
for i in "${!SOCKS[@]}"; do
  sock="${SOCKS[i]}" spec="${SPECS[i]}"
  mkdir -p "${spec#*=}" "$(dirname "${sock}")"
  dir="$(cd "${spec#*=}" && pwd)"
  if [[ "${resolved_dirs}" == *" ${dir} "* ]]; then
    echo "virtiofs: warning: ${dir} is shared more than once; the instances do not share caches" >&2
  fi
  resolved_dirs+="${dir} "
  rm -f "${sock}" "${sock}.pid"

  # The guest root maps to the invoking user, so files created in the guest
  # are owned by you on the host. 
  # --sandbox none avoids needing root or user namespaces.
  "${VIRTIOFSD_BIN}" \
    --socket-path="${sock}" \
    --shared-dir "${dir}" \
    --sandbox none \
    --log-level warn \
    --translate-uid="map:0:$(id -u):1" \
    --translate-gid="map:0:$(id -g):1" \
    </dev/null >"${sock%.sock}.log" 2>&1 &
  PIDS+=("$!")
done

# Wait (up to ~5 seconds per share) until every socket exists
for i in "${!SOCKS[@]}"; do
  sock="${SOCKS[i]}" tag="${SPECS[i]%%=*}" log="${SOCKS[i]%.sock}.log"
  for _ in $(seq 50); do
    [[ -S "${sock}" ]] && break
    kill -0 "${PIDS[i]}" 2>/dev/null || break
    sleep 0.1
  done
  if [[ ! -S "${sock}" ]]; then
    echo "virtiofs: virtiofsd for '${tag}' did not create ${sock}; see ${log}" >&2
    tail -n 20 "${log}" >&2 || true
    exit 1
  fi
  echo "virtiofs: sharing '${tag}' = ${SPECS[i]#*=} (log: ${log})" >&2
done

# Run the command as a child, not with exec, so that the EXIT trap still runs
# and stops the virtiofsd processes once the command is done.
"$@"
