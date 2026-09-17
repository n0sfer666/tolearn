#!/bin/sh
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
. "$root/scripts/lib/size.sh"

target=${TOLEARN_TARGET:-$root/target}
registry=${CARGO_HOME:-$HOME/.cargo}/registry
sources="$registry/src"
archives="$registry/cache"
days=${1:-7}

case $days in
    ''|*[!0-9]*) printf 'порог в сутках — целое число не меньше 1\n' >&2; exit 2 ;;
esac
[ "$days" -ge 1 ] || { printf 'порог в сутках — целое число не меньше 1\n' >&2; exit 2; }
older=$((days - 1))

if [ ! -f "$target/CACHEDIR.TAG" ]; then
    printf 'не дерево сборки cargo (нет CACHEDIR.TAG): %s\n' "$target" >&2
    exit 2
fi

printf 'дерево сборки: %s\n' "$target"
if [ -f "$registry/CACHEDIR.TAG" ]; then
    printf 'исходники крейтов: %s\n' "$sources"
else
    printf 'реестр крейтов пропускаю, нет %s/CACHEDIR.TAG\n' "$registry" >&2
    sources=
fi

fail=0
kept=0
list=$(mktemp)
trap 'rm -f "$list"' EXIT

was_target=$(kilobytes "$target")
was_sources=$(kilobytes "$sources")

find "$target" -mindepth 2 -maxdepth 3 -type d -name incremental -prune >"$list" || fail=1
while IFS= read -r dir; do
    printf '== кэш пересборок %s\n' "${dir#"$target/"}"
    rm -rf "$dir" || fail=1
done <"$list"

for kind in deps build .fingerprint; do
    find "$target" -mindepth 2 -maxdepth 3 -type d -name "$kind" -prune >"$list" || fail=1
    while IFS= read -r dir; do
        printf '== старше %s суток в %s\n' "$days" "${dir#"$target/"}"
        find "$dir" -mindepth 1 -maxdepth 1 -mtime +"$older" -exec rm -rf {} + || fail=1
    done <"$list"
done

if [ -n "$sources" ] && [ -d "$sources" ]; then
    find "$sources" -mindepth 2 -maxdepth 2 -type d -mtime +"$older" >"$list" || fail=1
    while IFS= read -r dir; do
        index=$(basename "$(dirname "$dir")")
        if [ ! -f "$archives/$index/$(basename "$dir").crate" ]; then
            kept=$((kept + 1))
            continue
        fi
        printf '== исходники крейта %s\n' "${dir#"$sources/"}"
        rm -rf "$dir" || fail=1
    done <"$list"
fi

now_target=$(kilobytes "$target")
now_sources=$(kilobytes "$sources")

printf 'target: было %s, стало %s\n' "$(human "$was_target")" "$(human "$now_target")"
printf 'исходники крейтов: было %s, стало %s\n' "$(human "$was_sources")" "$(human "$now_sources")"
printf 'освободилось %s\n' "$(human "$((was_target - now_target + was_sources - now_sources))")"
if [ "$kept" -gt 0 ]; then
    printf 'крейтов без архива .crate (не тронуты, без сети не вернуть): %s\n' "$kept"
fi
printf 'из архивов .crate исходники распакуются без сети\n'
printf 'удалённое из target вернётся при следующей сборке, только она будет дольше\n'

[ "$fail" = 0 ] || { printf 'не всё удалилось, смотрите сообщения выше\n' >&2; exit 1; }
