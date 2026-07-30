#!/bin/sh
set -eu

route=${1:?путь экрана, например /ru/graph/}
out=${2:-shot.png}
size=${3:-1280,900}
wait=${SHOT_WAIT:-40}
host=${TOLEARN_UI:-http://localhost:4321}
chrome=${CHROME:-/Applications/Google Chrome.app/Contents/MacOS/Google Chrome}
profile=$(mktemp -d "${TMPDIR:-/tmp}/tolearn-shot.XXXXXX")

[ -x "$chrome" ] || { echo "нет Chrome: $chrome" >&2; exit 1; }
curl -sf -o /dev/null -m 5 "$host/" || { echo "не отвечает $host — подними pnpm -C ui dev" >&2; exit 1; }

case "$out" in
    *.html) mode=--dump-dom ;;
    *) mode="--screenshot=$out" ;;
esac

rm -rf "$profile" "$out"
if [ "$mode" = "--dump-dom" ]; then
    "$chrome" --headless=new --disable-gpu --hide-scrollbars --window-size="$size" \
        --virtual-time-budget=8000 --user-data-dir="$profile" \
        --dump-dom "$host$route" >"$out" 2>/dev/null &
else
    "$chrome" --headless=new --disable-gpu --hide-scrollbars --window-size="$size" \
        --virtual-time-budget=8000 --user-data-dir="$profile" \
        "$mode" "$host$route" >/dev/null 2>&1 &
fi
child=$!

spent=0
while [ ! -s "$out" ] && [ "$spent" -lt "$wait" ]; do
    sleep 1
    spent=$((spent + 1))
done
sleep 1
kill "$child" 2>/dev/null || true
wait "$child" 2>/dev/null || true
rm -rf "$profile"

[ -s "$out" ] || { echo "пусто за ${wait}с: $out" >&2; exit 1; }
echo "снимок: $out ($(wc -c <"$out") байт, ${spent}с)"
