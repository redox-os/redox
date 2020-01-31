#!/usr/bin/env bash
# Begin /usr/sbin/make-ca.sh
#
# Script to create OpenSSL certs directory, GnuTLS certificate bundle, NSS
# shared DB, and Java cacerts from upstream certdata.txt and local sources
# 
# Authors: DJ Lucas
#          Bruce Dubbs
#
# Changes:
#
# 20170425 - Use p11-kit format anchors
#          - Add CKA_NSS_MOZILLA_CA_POLICY attribute for p11-kit anchors
#          - Add clientAuth OpenSSL attribute and (currently unused) NSS
#            CKA_TRUST_CLIENT_AUTH
# 20170119 - Show trust bits on local certs
#          - Add version output for help2man
# 20161210 - Add note about --force swich when same version
# 20161126 - Add -D/--destdir switch
# 20161124 - Add -f/--force switch to bypass version check
#          - Add multiple switches to allow for alternate localtions
#          - Add help text
# 20161118 - Drop make-cert.pl script
#          - Add support for Java and NSSDB

# Set defaults
VERSION="20170425"
CERTDATA="certdata.txt"
PKIDIR="/etc/pki"
SSLDIR="/etc/ssl"
CERTUTIL="certutil"
KEYTOOL="keytool"
OPENSSL="openssl"
ANCHORDIR="${PKIDIR}/anchors"
CABUNDLE="${SSLDIR}/ca-bundle.crt"
CERTDIR="${SSLDIR}/certs"
KEYSTORE="${SSLDIR}/java/cacerts"
NSSDB="${PKIDIR}/nssdb"
LOCALDIR="${SSLDIR}/local"
DESTDIR=""

# Some data in the certs have UTF-8 characters
export LANG=en_US.utf8

TEMPDIR=$(mktemp -d)
WORKDIR="${TEMPDIR}/work"
WITH_NSS=1
WITH_JAVA=1
FORCE=0

function get_args(){
  while test -n "${1}" ; do
    case "${1}" in
      -C | --certdata)
        check_arg $1 $2
        CERTDATA="${2}"
        shift 2
      ;;
      -D | --destdir)
        check_arg $1 $2
        DESTDIR="${2}"
        shift 2
      ;;
      -P | --pkidir)
        check_arg $1 $2
        PKIDIR="${2}"
        ANCHORDIR="${PKIDIR}/anchors"
        NSSDB="${PKIDIR}/nssdb"
        echo "${@}" | grep -e "-a " -e "--anchordir" \
                           -e "-n " -e "--nssdb" > /dev/null
        if test "${?}" == "0"; then
          echo "Error! ${1} cannot be used with the -a/--anchordir or -n/--nssdb switches."
          echo ""
          exit 3
        fi
        shift 2
      ;;
      -S | --ssldir)
        check_arg $1 $2
        SSLDIR="${2}"
        CABUNDLE="${SSLDIR}/ca-bundle.crt"
        CERTDIR="${SSLDIR}/certs"
        KEYSTORE="${SSLDIR}/java/cacerts"
        LOCALDIR="${SSLDIR}/local"
        echo "${@}" | grep -e "-c " -e "--cafile" \
                           -e "-d " -e "--cadir"  \
                           -e "-j " -e "--javacerts" > /dev/null
        if test "${?}" == "0"; then
          echo "Error! ${1} cannot be used with the -c/--cafile, -d/--cadir, or"
          echo "-j/--javacerts switches."
          echo ""
          exit 3
        fi

        shift 2
      ;;
      -a | --anchordir)
        check_arg $1 $2
        ANCHORDIR="${2}"
        echo "${@}" | grep -e "-P " -e "--pkidir" > /dev/null
        if test "${?}" == "0"; then
          echo "Error! ${1} cannot be used with the -P/--pkidir switch."
          echo ""
          exit 3
        fi
        shift 2
      ;;
      -c | --cafile)
        check_arg $1 $2
        CABUNDLE="${2}"
        echo "${@}" | grep -e "-S " -e "--ssldir" > /dev/null
        if test "${?}" == "0"; then
          echo "Error! ${1} cannot be used with the -S/--ssldir switch."
          echo ""
          exit 3
        fi
        shift 2
      ;;
      -d | --cadir)
        check_arg $1 $2
        CADIR="${2}"
        if test "${?}" == "0"; then
          echo "Error! ${1} cannot be used with the -S/--ssldir switch."
          echo ""
          exit 3
        fi
        shift 2
      ;;
      -j | --javacerts)
        check_arg $1 $2
        KEYSTORE="${2}"
        if test "${?}" == "0"; then
          echo "Error! ${1} cannot be used with the -S/--ssldir switch."
          echo ""
          exit 3
        fi
        shift 2
      ;;
      -l | --localdir)
        check_arg $1 $2
        LOCALDIR="${2}"
        shift 2
      ;;
      -n | --nssdb)
        check_arg $1 $2
        NSSDB="${2}"
        echo "${@}" | grep -e "-P " -e "--pkidir" > /dev/null
        if test "${?}" == "0"; then
          echo "Error! ${1} cannot be used with the -P/--pkidir switch."
          echo ""
          exit 3
        fi
        shift 2
      ;;
      -k | --keytool)
        check_arg $1 $2
        KEYTOOL="${2}"
        shift 2
      ;;
      -s | --openssl)
        check_arg $1 $2
        OPENSSL="${2}"
        shift 2
      ;;
      -t | --certutil)
        check_arg $1 $2
        CERTUTIL="${2}"
        shift 2
      ;;
      -f | --force)
        FORCE="1"
        shift 1
      ;;
      -h | --help)
        showhelp
        exit 0
      ;;
      -v | --version)
        echo -e "$(basename ${0}) ${VERSION}\n"
        exit 0
      ;;
      *)
        showhelp
        exit 1
      ;;
    esac
  done
}

function check_arg(){
  echo "${2}" | grep -v "^-" > /dev/null
  if [ -z "$?" -o ! -n "$2" ]; then
    echo "Error:  $1 requires a valid argument."
    exit 1
  fi
}

function showhelp(){
  echo ""
  echo "`basename ${0}` converts certdata.txt (provided by the Mozilla Foundation)"
  echo "into a complete PKI distribution for use with LFS or like distributions."
  echo ""
  echo "        -C  --certdata   The certdata.txt file (provided by Mozilla)"
  echo "                         Default: ./certdata.txt"
  echo ""
  echo "        -D  --destdir    Change the output directory and use relative"
  echo "                         paths for all other values."
  echo "                         Default: unset"
  echo ""
  echo "        -P  --pkidir     The output PKI directory - Cannot be used with"
  echo "                         the -a/--anchordir or -n/--nssdb switches"
  echo "                         Default: /etc/pki"
  echo ""
  echo "        -S  --ssldir     The output SSL root direcotry - Cannot be used"
  echo "                         with the -c/--cafile, -d/--cadir, or"
  echo "                         -j/--javacerts switches"
  echo "                         Defualt: /etc/ssl"
  echo ""
  echo "        -a  --anchordir  The output directory for OpenSSL trusted"
  echo "                         CA certificates used as trust anchors."
  echo "                         Default: \$PKIDIR/anchors"
  echo ""
  echo "        -c  --cafile     The output filename for the PEM formated bundle"
  echo "                         Default: \$SSLDIR/ca-bundle.crt"
  echo ""
  echo "        -d  --cadir      The output directory for the OpenSSL trusted"
  echo "                         CA certificates"
  echo "                         Deault: \$SSLDIR/certs/"
  echo ""
  echo "        -j  --javacerts  The output path for the Java cacerts file"
  echo "                         Default: \$SSLDIR/java/cacerts"
  echo ""
  echo "        -l  --localdir   The path to a local set of OpenSSL trusted"
  echo "                         certificates to include in the output"
  echo "                         Default: \$SSLDIR/local"
  echo ""
  echo "        -n  --nssdb      The output path for the shared NSS DB"
  echo "                         Default: \$PKIDIR/nssdb"
  echo ""
  echo "        -k  --keytool    The path to the java keytool utility"
  echo ""
  echo "        -s  --openssl    The path to the openssl utility"
  echo ""
  echo "        -t  --certutil   The path the certutil utility"
  echo ""
  echo "        -f  --force      Force run, even if source is not newer"
  echo ""
  echo "        -h  --help       Show this help message and exit"
  echo ""
  echo "        -v  --version    Show version information and exit"
  echo ""
  echo "Example: `basename ${0}` -f -C ~/certdata.txt"
  echo ""
}

# Convert CKA_TRUST values to trust flags for certutil
function convert_trust(){
  case $1 in
    CKT_NSS_TRUSTED_DELEGATOR)
      echo "C"
    ;;
    CKT_NSS_NOT_TRUSTED)
      echo "p"
    ;;
    CKT_NSS_MUST_VERIFY_TRUST)
      echo ""
    ;;
  esac
}

function convert_trust_arg(){
  case $1 in
    C)
      case $2 in
        sa)
          echo "-addtrust serverAuth"
        ;;
        sm)
          echo "-addtrust emailProtection"
        ;;
        cs)
          echo "-addtrust codeSigning"
        ;;
        ca)
          echo "-addtrust clientAuth"
        ;;
      esac
    ;;
    p)
      case $2 in
        sa)
          echo "-addreject serverAuth"
        ;;
        sm)
          echo "-addreject emailProtection"
        ;;
        cs)
          echo "-addreject codeSigning"
        ;;
        ca)
          echo "-addreject clientAuth"
        ;;
      esac
    ;;
    *)
      echo ""
    ;;
  esac
}
    
# Define p11-kit ext value constants (see p11-kit API documentation)
get-p11-val() {
  case $1 in
    p11sasmcs)
      p11value="0%2a%06%03U%1d%25%01%01%ff%04 0%1e%06%08%2b%06%01%05%05%07%03%04%06%08%2b%06%01%05%05%07%03%01%06%08%2b%06%01%05%05%07%03%03"
    ;;

    p11sasm)
      p11value="0 %06%03U%1d%25%01%01%ff%04%160%14%06%08%2b%06%01%05%05%07%03%04%06%08%2b%06%01%05%05%07%03%01"
    ;;

    p11sacs)
      p11value="0 %06%03U%1d%25%01%01%ff%04%160%14%06%08%2b%06%01%05%05%07%03%01%06%08%2b%06%01%05%05%07%03%03"
    ;;

    p11sa)
      p11value="0%16%06%03U%1d%25%01%01%ff%04%0c0%0a%06%08%2b%06%01%05%05%07%03%01"
    ;;

    p11smcs)
      p11value="0 %06%03U%1d%25%01%01%ff%04%160%14%06%08%2b%06%01%05%05%07%03%04%06%08%2b%06%01%05%05%07%03%03"
    ;;

    p11sm)
      p11value="0%16%06%03U%1d%25%01%01%ff%04%0c0%0a%06%08%2b%06%01%05%05%07%03%04"
    ;;

    p11cs)
      p11value="0%16%06%03U%1d%25%01%01%ff%04%0c0%0a%06%08%2b%06%01%05%05%07%03%03"
    ;;

    p11)
      p11value="0%18%06%03U%1d%25%01%01%ff%04%0e0%0c%06%0a%2b%06%01%04%01%99w%06%0a%10"
    ;;
  esac
}

# Process command line arguments
get_args $@

if test ! -r "${CERTDATA}"; then
  echo "${CERTDATA} was not found. The certdata.txt file must be in the local"
  echo "directory, or speficied with the --certdata switch."
  exit 1
fi

test -f "${CERTUTIL}" || WITH_NSS=0
test -f "${KEYTOOL}" || WITH_JAVA=0

VERSION=$(grep CVS_ID "${CERTDATA}" | cut -d " " -f 8)

if test "${VERSION}x" == "x"; then
  echo "WARNING! ${CERTDATA} has no 'Revision' in CVS_ID"
  echo "Will run conversion unconditionally."
  sleep 2
  VERSION="$(date -u +%Y%m%d-%H%M)"
else
  if test "${FORCE}" == "1"; then
    echo "Output forced. Will run conversion unconditionally."
    sleep 2
  elif test "${DESTDIR}x" == "x"; then
    test -f "${CABUNDLE}" &&
    OLDVERSION=$(grep "^VERSION:" "${CABUNDLE}" | cut -d ":" -f 2)
  fi
fi

if test "${OLDVERSION}x" == "${VERSION}x"; then
  echo "No update required! Use --force to update anyway."
  exit 0
fi

mkdir -p "${TEMPDIR}"/{certs,ssl/{certs,java},pki/{nssdb,anchors},work}
cp "${CERTDATA}" "${WORKDIR}/certdata.txt"
pushd "${WORKDIR}" > /dev/null

if test "${WITH_NSS}" == "1"; then
  # Create a blank NSS DB
  "${CERTUTIL}" -N --empty-password -d "sql:${TEMPDIR}/pki/nssdb"
fi

# Get a list of starting lines for each cert
CERTBEGINLIST=`grep -n "^# Certificate" "${WORKDIR}/certdata.txt" | \
                      cut -d ":" -f1`

# Dump individual certs to temp file
for certbegin in ${CERTBEGINLIST}; do
  awk "NR==$certbegin,/^CKA_TRUST_STEP_UP_APPROVED/" "${WORKDIR}/certdata.txt" \
      > "${TEMPDIR}/certs/${certbegin}.tmp" 
done

unset CERTBEGINLIST certbegin

for tempfile in ${TEMPDIR}/certs/*.tmp; do
  # Get a name for the cert
  certname="$(grep "^# Certificate" "${tempfile}" | cut -d '"' -f 2)"

  # Determine certificate trust values for SSL/TLS, S/MIME, and Code Signing
  satrust="$(convert_trust `grep '^CKA_TRUST_SERVER_AUTH' ${tempfile} | \
                  cut -d " " -f 3`)"
  smtrust="$(convert_trust `grep '^CKA_TRUST_EMAIL_PROTECTION' ${tempfile} | \
                  cut -d " " -f 3`)"
  cstrust="$(convert_trust `grep '^CKA_TRUST_CODE_SIGNING' ${tempfile} | \
                  cut -d " " -f 3`)"
  # Not currently included in NSS certdata.txt
  #catrust="$(convert_trust `grep '^CKA_TRUST_CLIENT_AUTH' ${tempfile} | \
  #                cut -d " " -f 3`)"

  # Get args for OpenSSL trust settings
  saarg="$(convert_trust_arg "${satrust}" sa)"
  smarg="$(convert_trust_arg "${smtrust}" sm)"
  csarg="$(convert_trust_arg "${cstrust}" cs)"
  # Not currently included in NSS certdata.txt
  #caarg="$(convert_trust_arg "${catrust}" ca)"

  # Convert to a PEM formated certificate
  printf $(awk '/^CKA_VALUE/{flag=1;next}/^END/{flag=0}flag{printf $0}' \
  "${tempfile}") | "${OPENSSL}" x509 -text -inform DER -fingerprint \
  > tempfile.crt

  # Get individual values for certificates
  certkey="$(${OPENSSL} x509 -in tempfile.crt -noout -pubkey)"
  certcer="$(${OPENSSL} x509 -in tempfile.crt)"
  certtxt="$(${OPENSSL} x509 -in tempfile.crt -noout -text)"

  # Get p11-kit label, oid, and values
  p11label="$(grep -m1 "Issuer" ${tempfile} | grep -o CN=.*$ | \
              cut -d ',' -f 1 | sed 's@CN=@@')"

  # if distrusted at all, x-distrusted
  if test "${satrust}" == "p" -o "${smtrust}" == "p" -o "${cstrust}" == "p"
  then
      # if any distrusted, x-distrusted
      p11trust="x-distrusted: true"
      p11oid="1.3.6.1.4.1.3319.6.10.1"
      p11value="0.%06%0a%2b%06%01%04%01%99w%06%0a%01%04 0%1e%06%08%2b%06%01%05%05%07%03%04%06%08%2b%06%01%05%05%07%03%01%06%08%2b%06%01%05%05%07%03%03"
  else
      p11trust="trusted: true"
      p11oid="2.5.29.37"
      trustp11="p11"
      if test "${satrust}" == "C"; then
          trustp11="${trustp11}sa"
      fi
      if test "${smtrust}" == "C"; then
          trustp11="${trustp11}sm"
      fi
      if test "${cstrust}" == "C"; then
          trustp11="${trustp11}cs"
      fi
      get-p11-val "${trustp11}"
  fi

  # Get a hash for the cert
  keyhash=$("${OPENSSL}" x509 -noout -in tempfile.crt -hash)

  # Print information about cert
  echo "Certificate:  ${certname}"
  echo "Keyhash:      ${keyhash}"

  # Place certificate into trust anchors dir
  anchorfile="${TEMPDIR}/pki/anchors/${keyhash}.pem"
  echo "[p11-kit-object-v1]" >> "${anchorfile}"
  echo "label: \"${p11label}\"" >> "${anchorfile}"
  echo "class: x-certificate-extension" >> "${anchorfile}"
  echo "object-id: ${p11oid}" >> "${anchorfile}"
  echo "value: \"${p11value}\"" >> "${anchorfile}"
  echo "modifiable: false" >> "${anchorfile}"
  echo "${certkey}" >> "${anchorfile}"
  echo "" >> "${anchorfile}"
  echo "[p11-kit-object-v1]" >> "${anchorfile}"
  echo "label: \"${p11label}\"" >> "${anchorfile}"
  echo "${p11trust}" >> "${anchorfile}"
  echo "nss-mozilla-ca-policy: true" >> "${anchorfile}"
  echo "modifiable: false" >> "${anchorfile}"
  echo "${certcer}" >> "${anchorfile}"
  echo "${certtxt}" | sed 's@^@#@' >> "${anchorfile}"
  echo "Added to p11-kit anchor directory with trust '${satrust},${smtrust},${cstrust}'."
  
  
  # Import certificates trusted for SSL/TLS into the Java keystore and 
  # GnuTLS certificate bundle
  if test "${satrust}x" == "Cx"; then
    # Java keystore
    if test "${WITH_JAVA}" == "1"; then
      "${KEYTOOL}" -import -noprompt -alias "${certname}"   \
                   -keystore "${TEMPDIR}/ssl/java/cacerts"  \
                   -storepass 'changeit' -file tempfile.crt \
      2>&1> /dev/null | \
      sed -e 's@Certificate was a@A@' -e 's@keystore@Java keystore.@'
    fi

    # GnuTLS certificate bundle
    cat tempfile.crt >> "${TEMPDIR}/ssl/ca-bundle.crt.tmp"
    echo "Added to GnuTLS ceritificate bundle."
  fi

  # Import certificate into the temporary certificate directory with
  # trust arguments
  "${OPENSSL}" x509 -in tempfile.crt -text -fingerprint \
      -setalias "${certname}" ${saarg} ${smarg} ${csarg}    \
      > "${TEMPDIR}/ssl/certs/${keyhash}.pem"
  echo "Added to OpenSSL certificate directory with trust '${satrust},${smtrust},${cstrust}'."

  # Import all certificates with trust args to the temporary NSS DB
  if test "${WITH_NSS}" == "1"; then
    "${CERTUTIL}" -d "sql:${TEMPDIR}/pki/nssdb" -A \
                  -t "${satrust},${smtrust},${cstrust}" \
                  -n "${certname}" -i tempfile.crt
    echo "Added to NSS shared DB with trust '${satrust},${smtrust},${cstrust}'."
  fi

  # Clean up the directory and environment as we go
  rm -f tempfile.crt
  unset keyhash subject certname
  unset satrust smtrust cstrust catrust sarg smarg csarg caarg
  unset p11trust p11oid p11value trustp11 certkey certcer certtxt
  echo -e "\n"
done
unset tempfile

# Sanity check
count=$(ls "${TEMPDIR}"/ssl/certs/*.pem | wc -l)
# Historically there have been between 152 and 165 certs
# A minimum of 140 should be safe for a rudimentry sanity check
if test "${count}" -lt "140" ; then
    echo "Error! Only ${count} certificates were generated!"
    echo "Exiting without update!"
    echo ""
    echo "${TEMPDIR} is the temporary working directory"
    exit 2
fi
unset count

# Generate the bundle
bundlefile=`basename "${CABUNDLE}"`
bundledir=`echo "${CABUNDLE}" | sed "s@/${bundlefile}@@"`
install -vdm755 "${DESTDIR}${bundledir}" 2>&1>/dev/null
test -f "${DESTDIR}${CABUNDLE}" && mv "${DESTDIR}${CABUNDLE}" \
                                      "${DESTDIR}${CABUNDLE}.old"
echo "VERSION:${VERSION}" > "${DESTDIR}${CABUNDLE}"
cat "${TEMPDIR}/ssl/ca-bundle.crt.tmp" >> "${DESTDIR}${CABUNDLE}" &&
rm -f "${DESTDIR}${CABUNDLE}.old"
unset bundlefile bundledir

# Install Java Cacerts
if test "${WITH_JAVA}" == "1"; then
  javafile=`basename "${KEYSTORE}"`
  javadir=`echo "${KEYSTORE}" | sed "s@/${javafile}@@"`
  install -vdm755 "${DESTDIR}${javadir}" 2>&1>/dev/null
  test -f "${DESTDIR}${KEYSTORE}" && mv "${DESTDIR}${KEYSTORE}" \
                                        "${DESTDIR}${KEYSTORE}.old"
  install -m644 "${TEMPDIR}/ssl/java/cacerts" "${DESTDIR}${KEYSTORE}" &&
  rm -f "${DESTDIR}${KEYSTORE}.old"
  unset javafile javadir
fi

# Install NSS Shared DB
if test "${WITH_NSS}" == "1"; then
  sed -e "s@${TEMPDIR}/pki/nssdb@${NSSDB}@"              \
      -e 's/library=/library=libnsssysinit.so/'          \
      -e 's/Flags=internal/Flags=internal,moduleDBOnly/' \
      -i "${TEMPDIR}/pki/nssdb/pkcs11.txt" 
  test -d "${DESTDIR}${NSSDB}" && mv "${DESTDIR}${NSSDB}" \
                                     "${DESTDIR}${NSSDB}.old"
  install -dm755 "${DESTDIR}${NSSDB}" 2>&1>/dev/null
  install -m644 "${TEMPDIR}"/pki/nssdb/{cert9.db,key4.db,pkcs11.txt} \
                 "${DESTDIR}${NSSDB}" &&
  rm -rf "${DESTDIR}${NSSDB}.old"
fi

# Install anchors in $ANCHORDIR
test -d "${DESTDIR}${ANCHORDIR}" && mv "${DESTDIR}${ANCHORDIR}"\
                                       "${DESTDIR}${ANCHORDIR}.old"
install -dm755 "${DESTDIR}${ANCHORDIR}" 2>&1>/dev/null
install -m644 "${TEMPDIR}"/pki/anchors/*.pem "${DESTDIR}${ANCHORDIR}" &&
rm -rf "${DESTDIR}${ANCHORDIR}.old"

# Install certificates in $CERTDIR
test -d "${DESTDIR}${CERTDIR}" && mv "${DESTDIR}${CERTDIR}" \
                                     "${DESTDIR}${CERTDIR}.old"
install -dm755 "${DESTDIR}${CERTDIR}" 2>&1>/dev/null
install -m644 "${TEMPDIR}"/ssl/certs/*.pem "${DESTDIR}${CERTDIR}" &&
rm -rf "${DESTDIR}${CERTDIR}.old"

# Import any certs in $LOCALDIR
# Don't do any checking, just trust the admin
if test -d "${LOCALDIR}"; then
  for cert in `find "${LOCALDIR}" -name "*.pem"`; do
    # Get some information about the certificate
    keyhash=$("${OPENSSL}" x509 -noout -in "${cert}" -hash)
    subject=$("${OPENSSL}" x509 -noout -in "${cert}" -subject)
    count=1
    while test "${count}" -lt 10; do
      echo "${subject}" | cut -d "/" -f "${count}" | grep "CN=" >/dev/null \
           && break
      let count++
    done
    certname=$(echo "${subject}" | cut -d "/" -f "${count}" | sed 's@CN=@@')

    echo "Certificate:  ${certname}"
    echo "Keyhash:      ${keyhash}"

    # Get trust information
    trustlist=$("${OPENSSL}" x509 -in "${cert}" -text -trustout | \
                       grep -A1 "Trusted Uses")
    satrust=""
    smtrust=""
    cstrust=""
    catrust=""
    satrust=$(echo "${trustlist}" | \
              grep "TLS Web Server" 2>&1> /dev/null && echo "C")
    smtrust=$(echo "${trustlist}" | \
              grep "E-mail Protection" 2>&1 >/dev/null && echo "C")
    cstrust=$(echo "${trustlist}" | \
              grep "Code Signing" 2>&1 >/dev/null && echo "C")
    catrust=$(echo "${trustlist}" | \
              grep "Client Auth" 2>&1 >/dev/null && echo "C")

    # Get reject information
    rejectlist=$("${OPENSSL}" x509 -in "${cert}" -text -trustout | \
                     grep -A1 "Rejected Uses")
    if test "${satrust}" == ""; then satrust=$(echo "${rejectlist}" | \
              grep "TLS Web Server" 2>&1> /dev/null && echo "p"); fi
    if test "${smtrust}" == ""; then smtrust=$(echo "${rejectlist}" | \
              grep "E-mail Protection" 2>&1> /dev/null && echo "p"); fi
    if test "${cstrust}" == ""; then cstrust=$(echo "${rejectlist}" | \
              grep "Code Signing" 2>&1> /dev/null && echo "p"); fi
    if test "${catrust}" == ""; then catrust=$(echo "${rejectlist}" | \
              grep "Client Auth" 2>&1> /dev/null && echo "p"); fi


    # Place certificate into trust anchors dir
    p11label="$(grep -m1 "Issuer" ${cert} | grep -o CN=.*$ | \
                cut -d ',' -f 1 | sed 's@CN=@@')"

    # if distrusted at all, x-distrusted
    if test "${satrust}" == "p" -o "${smtrust}" == "p" -o "${cstrust}" == "p"
    then
        # if any distrusted, x-distrusted
        p11trust="x-distrusted: true"
        p11oid="1.3.6.1.4.1.3319.6.10.1"
        p11value="0.%06%0a%2b%06%01%04%01%99w%06%0a%01%04 0%1e%06%08%2b%06%01%05%05%07%03%04%06%08%2b%06%01%05%05%07%03%01%06%08%2b%06%01%05%05%07%03%03"
    else
        p11trust="trusted: true"
        p11oid="2.5.29.37"
        trustp11="p11"
        if test "${satrust}" == "C"; then
            trustp11="${trustp11}sa"
        fi
        if test "${smtrust}" == "C"; then
            trustp11="${trustp11}sm"
        fi
        if test "${cstrust}" == "C"; then
            trustp11="${trustp11}cs"
        fi
        get-p11-val "${trustp11}"
    fi

    anchorfile="${DESTDIR}${ANCHORDIR}/${keyhash}.pem"

    echo "[p11-kit-object-v1]" >> "${anchorfile}"
    echo "label: \"${p11label}\"" >> "${anchorfile}"
    echo "class: x-certificate-extension" >> "${anchorfile}"
    echo "object-id: ${p11oid}" >> "${anchorfile}"
    echo "value: \"${p11value}\"" >> "${anchorfile}"
    echo "modifiable: false" >> "${anchorfile}"
    echo "${certkey}" >> "${anchorfile}"
    echo "" >> "${anchorfile}"
    echo "[p11-kit-object-v1]" >> "${anchorfile}"
    echo "label: \"${p11label}\"" >> "${anchorfile}"
    echo "${p11trust}" >> "${anchorfile}"
    echo "modifiable: false" >> "${anchorfile}"
    echo "${certcer}" >> "${anchorfile}"
    echo "${certtxt}" | sed 's@^@#@' >> "${anchorfile}"
    echo "Added to p11-kit anchor directory with trust '${satrust},${smtrust},${cstrust}'."

    # Install in Java keystore
    if test "${WITH_JAVA}" == "1" -a "${satrust}x" == "Cx"; then
      "${KEYTOOL}" -import -noprompt -alias "${certname}"                  \
                   -keystore "${DESTDIR}${KEYSTORE}"                       \
                   -storepass 'changeit' -file "${cert}" 2>&1> /dev/null | \
      sed -e 's@Certificate was a@A@' -e 's@keystore@Java keystore.@'
    fi

    # Append to the bundle - source should have trust info, process with
    # openssl x509 to strip
    if test "${satrust}x" == "Cx"; then
      "${OPENSSL}" x509 -in "${cert}" -text -fingerprint \
           >> "${DESTDIR}${CABUNDLE}"
      echo "Added to GnuTLS certificate bundle."
    fi

    # Install into OpenSSL certificate store
    "${OPENSSL}" x509 -in "${cert}" -text -fingerprint \
                      -setalias "${certname}"          \
                      >> "${DESTDIR}${CERTDIR}/${keyhash}.pem"
    echo "Added to OpenSSL certificate directory with trust '${satrust},${smtrust},${cstrust},${catrust}'."

    # Add to Shared NSS DB
    if test "${WITH_NSS}" == "1"; then
      "${OPENSSL}" x509 -in "${cert}" -text -fingerprint | \
      "${CERTUTIL}" -d "sql:${DESTDIR}${NSSDB}" -A                   \
                    -t "${satrust},${smtrust},${cstrust}"  \
                    -n "${certname}"
      echo "Added to NSS shared DB with trust '${satrust},${smtrust},${cstrust}'."
    fi

    unset keyhash subject count certname
    unset trustlist rejectlist satrust smtrust cstrust catrust
    unset p11trust p11oid p11value trustp11 certkey certcer certtxt
    echo ""

  done
  unset cert
fi

c_rehash "${DESTDIR}${CERTDIR}" 2>&1>/dev/null
popd > /dev/null

# Clean up the mess
rm -rf "${TEMPDIR}"

# End /usr/sbin/make-ca.sh
