#!/bin/bash
# vim: set noexpandtab:
# Adds modules from the given directory to index.ts, recursively
# TODO Delete after modules conversion

set -e

INDEX="index.ts"

if [ ! -f $INDEX ]; then
    echo "no index!"
    exit 1
fi

find "$1" -type f -print0 | while IFS= read -r -d '' file; do
    echo "import { ### } from \"./$file\";" >> $INDEX
    echo "!!!  $file" >> $INDEX
done

$EDITOR -s /dev/stdin <<<"
qqqqqgg?###
ddggPf.;hvbyF#viwp:s/\\.ts//
@qq@q:%s;!!!.*/\\([A-z0-9]*\\)\\.ts;  \\1,;
" $INDEX
