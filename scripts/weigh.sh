#!/bin/sh
set -eu

bundle=${1:-target/release/bundle}
mib=1048576
found=0
status=0

weigh() {
    kind=$1
    limit=$2
    for file in "$bundle/$kind"/*."$kind"; do
        [ -f "$file" ] || continue
        found=$((found + 1))
        size=$(wc -c <"$file")
        size=$((size))
        tenths=$((size * 10 / mib))
        printf '%s — %s.%s МБ при потолке %s МБ\n' \
            "$(basename "$file")" "$((tenths / 10))" "$((tenths % 10))" "$limit"
        if [ "$size" -gt "$((limit * mib))" ]; then
            echo "перевес: $(basename "$file") больше $limit МБ" >&2
            status=1
        fi
    done
}

weigh dmg 12
weigh msi 14
weigh deb 14

if [ "$found" -eq 0 ]; then
    echo "нечего взвешивать: в $bundle нет ни dmg, ни msi, ни deb" >&2
    exit 1
fi

exit "$status"
