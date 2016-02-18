#!/bin/bash
cargo count --unsafe-statistics $*
for file in `find $* -type f`
do
    UNSAFE=`cargo count --unsafe-statistics $file | grep Rust | tr -s ' ' | cut -d ' ' -f8`
    if [ -n "$UNSAFE" ]
    then
        echo -e "$UNSAFE\t$file"
    fi
done

