#!/bin/bash
CMDLN=$(eval 'perl parsc.pl $*')
#(>&2 echo -e "\e[34mCOMMANDLINE: $CMDLN")
RUST_BACKTRACE=1 rustc -L build/kernel $CMDLN
