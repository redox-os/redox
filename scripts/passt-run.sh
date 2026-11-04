#!/usr/bin/env bash
# Runs a command next to a passt vhost-user network backend, for `make ch`.
#   passt-run.sh <socket>.sock [passt options...] -- <command...>
# Under SELinux, passt may only use sockets in $XDG_RUNTIME_DIR or /tmp.

set -euo pipefail

STARTUP_TIMEOUT=5 # seconds

die() {
  echo "passt: $*" >&2
  exit 1
}

usage() {
  echo "usage: $0 <socket>.sock [passt options...] -- <command...>" >&2
  exit 2
}

[[ $# -ge 1 ]] || usage
SOCK="$1"
shift
PASST_ARGS=()
while [[ $# -gt 0 && "$1" != "--" ]]; do
  PASST_ARGS+=("$1")
  shift
done
[[ $# -ge 2 ]] || usage
shift # the "--"

[[ "${SOCK}" == *.sock ]] || die "socket '${SOCK}' must end in .sock"
[[ ${#SOCK} -lt 108 ]] || die "socket path too long (${#SOCK} bytes): ${SOCK}"
LOG="${SOCK%.sock}.log"
REPAIR_SOCK="${SOCK}.repair"

PASST_BIN="${PASST:-passt}"
command -v "${PASST_BIN}" >/dev/null 2>&1 \
  || die "passt not found. Install the passt package (dnf, apt and pacman all call it passt) or set PASST=/path/to/passt"
"${PASST_BIN}" --help 2>&1 | grep -q -- '--vhost-user' \
  || die "${PASST_BIN} has no --vhost-user mode, it is too old. Install a passt from 2025 or later."

socket_in_use() {
  ss -Hxl | awk -v sock="${SOCK}" '$5 == sock { found = 1 } END { exit !found }'
}

if socket_in_use; then
  die "${SOCK} is in use by another passt. Stop that VM first, or choose another socket."
fi

PASST_PID=

cleanup() {
  trap - EXIT INT TERM
  if [[ -n "${PASST_PID}" ]]; then
    kill "${PASST_PID}" 2>/dev/null || true
    wait "${PASST_PID}" 2>/dev/null || true
  fi
  rm -f "${SOCK}" "${REPAIR_SOCK}"
}

trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

start_passt() {
  mkdir -p "$(dirname "${SOCK}")"
  rm -f "${SOCK}" "${REPAIR_SOCK}"
  "${PASST_BIN}" --vhost-user --foreground --one-off --socket "${SOCK}" "${PASST_ARGS[@]}" \
    </dev/null >"${LOG}" 2>&1 &
  PASST_PID=$!
}

start_failed() {
  if [[ -s "${LOG}" ]]; then
    # After a bad option passt prints its whole usage text.
    sed '/^Usage:/,$d' "${LOG}" | tail -n 20 >&2
    die "$1; see ${LOG}"
  fi
  die "$1 and logged nothing. If SELinux is enforcing, put the socket in \$XDG_RUNTIME_DIR or /tmp."
}

wait_for_socket() {
  local deadline=$((SECONDS + STARTUP_TIMEOUT))
  until [[ -S "${SOCK}" ]]; do
    kill -0 "${PASST_PID}" 2>/dev/null || start_failed "passt exited"
    ((SECONDS < deadline)) || start_failed "passt did not create ${SOCK} within ${STARTUP_TIMEOUT}s"
    sleep 0.1
  done
}

start_passt
wait_for_socket
echo "passt: user-mode network at ${SOCK} (log: ${LOG})" >&2

# Not exec'd, so cleanup runs when the command ends.
"$@"
