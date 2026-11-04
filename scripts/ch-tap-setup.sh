#!/usr/bin/env bash
# Host setup for `make ch net=tap`: tap redox0, IPv4 forwarding, firewalld NAT
# and dnsmasq DHCP/DNS. Needs firewalld and dnsmasq. Re-run after a reboot.
# As root without sudo, set TAP_USER=<user>.

set -euo pipefail

# Debian does not put the sbin directories in a normal user's PATH.
PATH="${PATH}:/usr/local/sbin:/usr/sbin:/sbin"

TAP=redox0
GUEST_MAC=52:54:00:12:34:56
TAP_ADDR=192.168.249.1
TAP_CIDR="${TAP_ADDR}/24"
DHCP_FIRST=192.168.249.10
DHCP_LAST=192.168.249.50
DHCP_RANGE="${DHCP_FIRST},${DHCP_LAST},255.255.255.0,12h"
ZONE=chnet
POLICY=chnet-nat
SYSCTL_DROPIN=/etc/sysctl.d/90-redox-tap.conf
DNSMASQ_PID=/run/redox0-dnsmasq.pid
WAIT_TIMEOUT=3 # seconds

step() {
    printf '\n==> %s\n' "$*"
}

info() {
    printf '    %s\n' "$*"
}

die() {
    echo "error: $*" >&2
    exit 1
}

as_root() {
    if [[ "${EUID}" -eq 0 ]]; then
        "$@"
    else
        sudo "$@"
    fi
}

wait_until() {
    local deadline=$((SECONDS + $1))
    shift
    until "$@"; do
        ((SECONDS < deadline)) || return 1
        sleep 0.1
    done
}

# Only tun/tap devices have tun_flags. IFF_TAP is 0x0002, IFF_UP is 0x1,
# owner -1 means none.
tap_exists() {
    [[ -e "/sys/class/net/${TAP}" ]]
}

is_tap() {
    [[ -r "/sys/class/net/${TAP}/tun_flags" ]] \
        && (($(<"/sys/class/net/${TAP}/tun_flags") & 0x0002))
}

tap_owner() {
    cat "/sys/class/net/${TAP}/owner"
}

link_is_up() {
    (($(<"/sys/class/net/${TAP}/flags") & 0x1))
}

tap_addrs() {
    ip -o -4 addr show dev "${TAP}" | awk '{ print $4 }'
}

has_addr() {
    tap_addrs | grep -qx "${TAP_CIDR}"
}

forwarding_on() {
    [[ "$(sysctl -n net.ipv4.ip_forward)" == "1" ]]
}

forwarding_persisted_by() {
    grep -RlsE '^[[:space:]]*net\.ipv4\.ip_forward[[:space:]]*=[[:space:]]*1[[:space:]]*(#.*)?$' \
        /etc/sysctl.conf /etc/sysctl.d /usr/lib/sysctl.d | head -n 1
}

FW_CHANGED=0

fw_ensure_exists() {
    local kind="$1" name="$2" list
    case "${kind}" in
        zone) list=--get-zones ;;
        policy) list=--get-policies ;;
    esac
    if firewall-cmd --permanent "${list}" | tr ' ' '\n' | grep -qx "${name}"; then
        info "${kind} ${name}: exists"
    else
        info "${kind} ${name}: creating"
        as_root firewall-cmd --permanent "--new-${kind}=${name}" >/dev/null
        FW_CHANGED=1
    fi
}

fw_ensure() {
    local what="$1" scope="$2" query="$3" add="$4"
    if firewall-cmd --permanent "${scope}" "${query}" >/dev/null; then
        info "${what}: already set"
    else
        info "${what}: adding"
        as_root firewall-cmd --permanent "${scope}" "${add}" >/dev/null
        FW_CHANGED=1
    fi
}

# --get-target is permanent-only. The runtime value is the target: line.
policy_accepts() {
    [[ "$(firewall-cmd --policy="${POLICY}" --list-all | awk '$1 == "target:" { print $2 }')" == "ACCEPT" ]]
}

dnsmasq_pids() {
    pgrep -f "(^|/)dnsmasq .*--interface=${TAP}( |$)" || true
}

# dnsmasq binds DHCP to 0.0.0.0%redox0:67 but DNS to 192.168.249.1:53.
dhcp_listening() {
    ss -H -uln | awk -v t="${TAP}" '$4 ~ ("%" t ":67$") { f = 1 } END { exit !f }'
}

dns_listening() {
    ss -H -uln | awk -v a="${TAP_ADDR}" -v t="${TAP}" \
        '$4 == (a ":53") || $4 == (a "%" t ":53") { f = 1 } END { exit !f }'
}

dnsmasq_answering() {
    dhcp_listening && dns_listening
}

dhcp_port_free() {
    ! dhcp_listening
}

preflight() {
    if [[ -z "${TAP_USER:-}" ]]; then
        if [[ "${EUID}" -eq 0 ]]; then
            TAP_USER="${SUDO_USER:-}"
        else
            TAP_USER="$(id -un)"
        fi
    fi
    if [[ -z "${TAP_USER}" || "${TAP_USER}" == "root" ]]; then
        die "run this as the user that starts cloud-hypervisor (sudo from that user is fine), or set TAP_USER=<user>."
    fi
    TAP_UID="$(id -u "${TAP_USER}")" || die "no such user: ${TAP_USER}"

    if ! command -v firewall-cmd >/dev/null 2>&1 || ! firewall-cmd --state >/dev/null 2>&1; then
        die "firewalld is not running. This script only supports firewalld; without it, use plain 'make ch', whose passt network needs no tap."
    fi
    local cmd
    for cmd in ip sysctl dnsmasq ss pgrep; do
        command -v "${cmd}" >/dev/null 2>&1 || die "missing command: ${cmd}"
    done

    echo "Setting up ${TAP} for 'make ch net=tap': host ${TAP_CIDR}, guest DHCP pool ${DHCP_FIRST}-${DHCP_LAST}."
    if [[ "${EUID}" -ne 0 ]]; then
        echo "Privileged steps run through sudo, which may ask you to authenticate."
    fi
}

setup_tap() {
    step "Tap device ${TAP}"
    TAP_IS_NEW=0
    if tap_exists; then
        is_tap || die "${TAP} exists but is not a tap device. Remove or rename it first."
        local owner
        owner="$(tap_owner)"
        if [[ "${owner}" == "${TAP_UID}" ]]; then
            info "exists and is owned by ${TAP_USER} (uid ${TAP_UID}), keeping it"
            return
        fi
        if [[ "${owner}" == "-1" ]]; then
            info "exists but has no owner, not ${TAP_USER} (uid ${TAP_UID}): deleting it"
        else
            info "exists but is owned by uid ${owner}, not ${TAP_USER} (uid ${TAP_UID}): deleting it"
        fi
        as_root ip link del "${TAP}"
    else
        info "does not exist"
    fi
    info "creating it, owned by ${TAP_USER} so cloud-hypervisor can open it without root"
    as_root ip tuntap add "${TAP}" mode tap user "${TAP_USER}"
    TAP_IS_NEW=1
}

setup_address() {
    step "Host address ${TAP_CIDR} on ${TAP} (the guest's gateway and DNS server)"
    local addr
    for addr in $(tap_addrs); do
        [[ "${addr}" == "${TAP_CIDR}" ]] || die "${TAP} already has ${addr}; this script expects ${TAP_CIDR}."
    done
    if has_addr; then
        info "already set"
    else
        info "adding it"
        as_root ip addr add "${TAP_CIDR}" dev "${TAP}"
    fi
    if link_is_up; then
        info "link is already up"
    else
        info "setting the link up"
        as_root ip link set "${TAP}" up
    fi
}

setup_forwarding() {
    step "IPv4 forwarding (routes guest packets out of the host)"
    if forwarding_on; then
        info "net.ipv4.ip_forward is already 1"
    else
        info "setting net.ipv4.ip_forward=1"
        as_root sysctl -q -w net.ipv4.ip_forward=1
    fi

    local persist
    persist="$(forwarding_persisted_by || true)"
    if [[ -n "${persist}" ]]; then
        info "kept across reboots by ${persist}"
    else
        info "writing ${SYSCTL_DROPIN} so it survives a reboot"
        printf 'net.ipv4.ip_forward = 1\n' | as_root tee "${SYSCTL_DROPIN}" >/dev/null
    fi
}

setup_firewalld() {
    step "firewalld zone ${ZONE} (lets the guest reach dnsmasq on the host)"
    fw_ensure_exists zone "${ZONE}"
    fw_ensure "interface ${TAP}" --zone="${ZONE}" --query-interface="${TAP}" --add-interface="${TAP}"
    fw_ensure "service dhcp (UDP 67)" --zone="${ZONE}" --query-service=dhcp --add-service=dhcp
    fw_ensure "service dns (port 53)" --zone="${ZONE}" --query-service=dns --add-service=dns

    step "firewalld policy ${POLICY} (forwards and masquerades guest traffic to the outside)"
    fw_ensure_exists policy "${POLICY}"
    fw_ensure "ingress zone ${ZONE}" --policy="${POLICY}" --query-ingress-zone="${ZONE}" --add-ingress-zone="${ZONE}"
    fw_ensure "egress zone ANY" --policy="${POLICY}" --query-egress-zone=ANY --add-egress-zone=ANY
    if [[ "$(firewall-cmd --permanent --policy="${POLICY}" --get-target)" == "ACCEPT" ]]; then
        info "target ACCEPT: already set"
    else
        info "target ACCEPT: setting"
        as_root firewall-cmd --permanent --policy="${POLICY}" --set-target=ACCEPT >/dev/null
        FW_CHANGED=1
    fi
    fw_ensure "masquerade" --policy="${POLICY}" --query-masquerade --add-masquerade

    step "Applying firewalld settings"
    if [[ "${FW_CHANGED}" -eq 1 ]]; then
        info "reloading firewalld to apply the permanent changes"
        as_root firewall-cmd --reload >/dev/null
    else
        info "nothing changed, no reload needed"
    fi
    # firewalld does not always attach an interface that appeared after it
    # started.
    if firewall-cmd --zone="${ZONE}" --query-interface="${TAP}" >/dev/null; then
        info "${TAP} is in zone ${ZONE} at runtime"
    else
        info "attaching ${TAP} to zone ${ZONE} at runtime (firewalld missed the new interface)"
        as_root firewall-cmd --zone="${ZONE}" --add-interface="${TAP}" >/dev/null
    fi
}

# dnsmasq --bind-interfaces keeps the old interface index, so a new tap needs
# a new dnsmasq.
setup_dnsmasq() {
    step "dnsmasq on ${TAP} (DHCP ${DHCP_FIRST}-${DHCP_LAST}, gateway and DNS ${TAP_ADDR})"
    local pids pid
    mapfile -t pids < <(dnsmasq_pids)

    if [[ "${TAP_IS_NEW}" -eq 0 ]] && dnsmasq_answering; then
        info "already answering DHCP and DNS (pid ${pids[*]:-unknown}), keeping it"
        return
    fi
    if [[ "${TAP_IS_NEW}" -eq 0 ]]; then
        info "dnsmasq is not answering both DHCP and DNS on ${TAP}"
    elif [[ ${#pids[@]} -gt 0 ]]; then
        info "${TAP} was recreated, so dnsmasq (pid ${pids[*]}) is bound to the old interface"
    fi

    for pid in "${pids[@]}"; do
        info "stopping dnsmasq pid ${pid}"
        as_root kill "${pid}" || true
    done
    if ! wait_until "${WAIT_TIMEOUT}" dhcp_port_free; then
        ss -uln | grep "${TAP}" >&2 || true
        die "UDP 67 on ${TAP} is still in use."
    fi

    info "starting dnsmasq (pid file ${DNSMASQ_PID})"
    # --conf-file=/dev/null: /etc/dnsmasq.conf must not add other interfaces.
    as_root dnsmasq \
        --conf-file=/dev/null \
        --interface="${TAP}" \
        --bind-interfaces \
        --except-interface=lo \
        --pid-file="${DNSMASQ_PID}" \
        --dhcp-range="${DHCP_RANGE}" \
        --dhcp-option=3,"${TAP_ADDR}" \
        --dhcp-option=6,"${TAP_ADDR}"
    wait_until "${WAIT_TIMEOUT}" dnsmasq_answering || true
}

CHECK_FAILED=0

check() {
    local what="$1"
    shift
    if "$@" >/dev/null 2>&1; then
        info "ok      ${what}"
    else
        info "FAILED  ${what}"
        CHECK_FAILED=1
    fi
}

run_checks() {
    step "Checking"
    check "${TAP} has ${TAP_CIDR}" has_addr
    check "net.ipv4.ip_forward is 1" forwarding_on
    check "${TAP} is in zone ${ZONE}" firewall-cmd --zone="${ZONE}" --query-interface="${TAP}"
    check "zone ${ZONE} allows dhcp" firewall-cmd --zone="${ZONE}" --query-service=dhcp
    check "zone ${ZONE} allows dns" firewall-cmd --zone="${ZONE}" --query-service=dns
    check "policy ${POLICY} target is ACCEPT" policy_accepts
    check "policy ${POLICY} masquerades" firewall-cmd --policy="${POLICY}" --query-masquerade
    check "dnsmasq answers DHCP on ${TAP} (UDP 67)" dhcp_listening
    check "dnsmasq answers DNS on ${TAP_ADDR} (UDP 53)" dns_listening

    if [[ "${CHECK_FAILED}" -ne 0 ]]; then
        echo >&2
        echo "Sockets on ${TAP}:" >&2
        ss -uln | grep -E "%${TAP}:|${TAP_ADDR}:" >&2 || true
        die "setup is incomplete, see the FAILED lines above."
    fi
}

print_summary() {
    echo
    echo "${TAP} is ready. It shows NO-CARRIER until cloud-hypervisor opens it; that is normal."
    echo "Start Redox with 'make ch net=tap'. It adds:"
    echo "  --net tap=${TAP},mac=${GUEST_MAC},offload_tso=off,offload_ufo=off,offload_csum=off"
    echo "Then optionally run 'dhcpd -v' in Redox and watch leases happen on the host with 'journalctl -t dnsmasq-dhcp -f'."
}

main() {
    preflight
    setup_tap
    setup_address
    setup_forwarding
    setup_firewalld
    setup_dnsmasq
    run_checks
    print_summary
}

main "$@"
