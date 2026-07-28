#!/bin/sh
set -eu

killed=$(pkill -f 'toLearn/target/debug/tolearn' >/dev/null 2>&1 && echo killed || echo none)
temp=${TMPDIR:-/tmp}
left=0

for path in "$temp"/tolearn-*; do
    [ -e "$path" ] || continue
    rm -rf "$path" || left=$((left + 1))
done

echo "процессы: $killed"
echo "каталоги: $temp/tolearn-* удалены, не поддалось: $left"
