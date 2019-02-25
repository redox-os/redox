GIT=https://gitlab.redox-os.org/redox-os/uutils.git
GIT_UPSTREAM=https://github.com/uutils/coreutils.git
CARGOFLAGS="--no-default-features --features redox --bin uutils -- -C lto"

BINS=(
  base32
  base64
  basename
  chmod
  cksum
  comm
  cp
  cut
  date
  dircolors
  dirname
  echo
  env
  expand
  expr
  factor
  false
  fmt
  fold
  head
  install
  link
  ls
  mktemp
  mv
  od
  paste
  printenv
  printf
  pwd
  readlink
  realpath
  relpath
  rm
  rmdir
  seq
  shuf
  sleep
  split
  sum
  tac
  tee
  tr
  true
  truncate
  tsort
  unexpand
  uniq
  wc
  yes
)

function recipe_stage {
    mkdir -p "$1/bin"
    for bin in "${BINS[@]}"
    do
      ln -s uutils "$1/bin/$bin"
    done
}
