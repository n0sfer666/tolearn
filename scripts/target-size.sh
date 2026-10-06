#!/bin/sh
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
. "$root/scripts/lib/size.sh"

target=${TOLEARN_TARGET:-$root/target}
limit=${TOLEARN_TARGET_LIMIT_GB:-10}

case $limit in
    ''|*[!0-9]*)
        printf 'TOLEARN_TARGET_LIMIT_GB — целое число гигабайт, а не «%s»\n' "$limit" >&2
        exit 2
        ;;
esac

[ -d "$target" ] || exit 0

kb=$(kilobytes "$target")
printf '== target: %s\n' "$(human "$kb")"

if [ "$kb" -ge $((limit * 1048576)) ]; then
    printf 'target перевалил за %s ГБ — пора почистить: make clean-target\n' "$limit" >&2
fi
