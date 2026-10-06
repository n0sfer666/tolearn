#!/bin/sh
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
prefix="$root/target/release/bundle/macos/rw."

if [ "$(uname -s)" != Darwin ]; then
    exit 0
fi

points=$(hdiutil info | awk -v prefix="$prefix" '
    /^image-path/ { ours = index($0, prefix) > 0 }
    ours { for (i = 1; i <= NF; i++) if ($i ~ /^\/Volumes\//) print $i }
')

for point in $points; do
    echo "== отключаю образ прошлого прогона: $point"
    if ! diskutil eject force "$point" >/dev/null 2>&1 && ! hdiutil detach "$point" -force >/dev/null 2>&1; then
        echo "не удалось отключить $point — отключите вручную: diskutil eject force $point" >&2
        exit 1
    fi
done

for image in "$prefix"*.dmg; do
    [ -f "$image" ] || continue
    echo "== убираю недоделанный образ $(basename "$image")"
    rm -f "$image"
done

for point in /Volumes/dmg.*; do
    [ -d "$point" ] || continue
    echo "внимание: подключён чужой образ $point — если упаковка .dmg сорвётся, отключите его: diskutil eject force $point" >&2
done
