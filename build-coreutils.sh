#!/bin/bash
# Build uutils coreutils with Cranelift for aarch64 Redox
set -e

cd /opt/other/redox/recipes/core/uutils/source

NIGHTLY="nightly-2026-01-02"
TARGET="/opt/other/redox/recipes/core/base/source/aarch64-unknown-redox-clif.json"
CRANELIFT="/opt/other/rustc_codegen_cranelift/dist/lib/librustc_codegen_cranelift.dylib"
RELIBC="/opt/other/redox/recipes/core/relibc/source/target/aarch64-unknown-redox-clif/release"

export DYLD_LIBRARY_PATH=~/.rustup/toolchains/${NIGHTLY}-aarch64-apple-darwin/lib

export RUSTFLAGS="-Zcodegen-backend=${CRANELIFT} \
  -Crelocation-model=static \
  -Clink-arg=-L${RELIBC} \
  -Clink-arg=${RELIBC}/crt0.o \
  -Clink-arg=${RELIBC}/crt0_rust.o \
  -Clink-arg=${RELIBC}/crti.o \
  -Clink-arg=${RELIBC}/crtn.o \
  -Clink-arg=-lunwind_stubs \
  -Clink-arg=-z -Clink-arg=muldefs \
  -Cpanic=abort"

echo "=== Building coreutils with Cranelift ==="
# Minimal feature set: essential utilities without C deps (no expr, no hashsum/b3sum)
cargo +${NIGHTLY} build \
    --target ${TARGET} \
    --release \
    --no-default-features \
    --features "base64,basename,cat,cp,cut,date,dd,dir,dirname,du,echo,env,false,fmt,fold,head,join,link,ln,ls,mkdir,mktemp,more,mv,nl,od,paste,pr,printenv,printf,pwd,readlink,realpath,rm,rmdir,seq,shuf,sleep,sort,split,tac,tail,tee,test,touch,tr,true,truncate,tsort,unexpand,uniq,unlink,vdir,wc,yes,chmod,stat,uname" \
    -Z build-std=core,alloc,std,panic_abort \
    -Zbuild-std-features=compiler_builtins/no-f16-f128 \
    -p coreutils

echo "=== Stripping ==="
llvm-strip -o /tmp/coreutils target/aarch64-unknown-redox-clif/release/coreutils

echo "=== Done ==="
ls -la /tmp/coreutils
file /tmp/coreutils
