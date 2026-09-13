#!/bin/sh
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
target=${CARGO_TARGET_DIR:-$root/target}
bridge=4319
ui=4321
chiptune=3f6c2a1e-8b4d-4c7a-9e21-5d0f7b3a6c84
nes=7a1d4e90-2c3b-4f58-8d6e-1b9a0c5e7f23
rom=e4c90b7a-15f2-4d6e-8b38-0a2c6f9d1e47

for port in "$bridge" "$ui"; do
    if lsof -nP -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1; then
        echo "порт $port занят — погаси прежний стенд или то, что его держит" >&2
        exit 1
    fi
done

cargo build -q -p tolearn-cli --bin tolearn
cargo build -q -p tolearn-app --example bridge

temp=${TMPDIR:-/tmp}
stand=$(mktemp -d "${temp%/}/tolearn-stand.XXXXXX")
export XDG_CONFIG_HOME="$stand/config" XDG_DATA_HOME="$stand/data"
data="$XDG_DATA_HOME/tolearn"
pids=

stop() {
    trap - EXIT INT TERM
    for pid in $pids; do
        kill -TERM -- "-$pid" 2>/dev/null || true
    done
    wait 2>/dev/null || true
    rm -rf "$stand"
    echo "стенд погашен"
}
trap stop EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
set -m

echo "стенд: $stand"
"$target/debug/examples/bridge" &
pids=$!

awaited() {
    spent=0
    until curl -s -o /dev/null -m 2 "$1"; do
        spent=$((spent + 1))
        [ "$spent" -lt "$2" ] || { echo "не отвечает $1 за $2 с" >&2; exit 1; }
        sleep 1
    done
}

ask() {
    curl -sf -o /dev/null -m 30 -H 'content-type: application/json' -d "$1" "http://127.0.0.1:$bridge/" ||
        { echo "мост отказал: $1" >&2; exit 1; }
}

awaited "http://127.0.0.1:$bridge/" 30
for source in examples/chiptune fixtures/v2/valid/nes-dev; do
    package="$stand/$(basename "$source").tolearn"
    "$target/debug/tolearn" pack "$root/$source" "$package" >/dev/null
    ask "{\"name\":\"import_package\",\"payload\":{\"path\":\"$package\"}}"
done
for fixture in "$root"/fixtures/v2/stand/*.yaml; do
    program=$(basename "$fixture" .yaml)
    mkdir -p "$data/state/$program"
    cp "$fixture" "$data/state/$program/state.yaml"
done

pnpm -C "$root/ui" dev --port "$ui" >"$stand/ui.log" 2>&1 &
pids="$pids $!"
awaited "http://localhost:$ui/" 60

host="http://localhost:$ui/ru"
cat <<EOF
экраны:
  библиотека        $host/
  программа         $host/program/?program=$chiptune
  этап: начат       $host/stage/?program=$chiptune&stage=voices
  развилка          $host/next/?program=$chiptune&stage=voices
  ветка nes-dev     $host/program/?program=$nes&node=$rom
  этап: зачёт сдан  $host/stage/?program=$nes&node=$rom&stage=first-rom
  этап: пропущен    $host/stage/?program=$nes&node=$rom&stage=linker
  новая программа   $host/new/
  поиск             $host/search/
  настройки         $host/settings/
снимок: SHOT_THEME=dark sh scripts/shot.sh '/ru/settings/' shot.png
Ctrl+C гасит мост и astro
EOF

while :; do
    for pid in $pids; do
        kill -0 "$pid" 2>/dev/null || { echo "процесс $pid стенда упал, лог astro: $stand/ui.log" >&2; tail -20 "$stand/ui.log" >&2; exit 1; }
    done
    sleep 1
done
