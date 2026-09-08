#!/bin/sh
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
variant=${1:-base}
destination=${2:-/Applications}

if [ "$(uname -s)" != Darwin ]; then
    echo "установка через .dmg есть только на macOS; на Windows и Linux ставится .msi или .deb — docs/ru/install/" >&2
    exit 1
fi

case "$variant" in
    base)
        product=tolearn
        set --
        ;;
    with-speech)
        product=tolearn-with-speech
        set -- --features speech --config tauri.with-speech.conf.json
        ;;
    *)
        echo "неизвестный вариант: $variant (нужен base или with-speech)" >&2
        exit 1
        ;;
esac

if ! command -v cargo-tauri >/dev/null 2>&1; then
    echo "нет tauri-cli: cargo install tauri-cli --version 2.11.4 --locked" >&2
    exit 1
fi

if pgrep -x tolearn-desktop >/dev/null 2>&1; then
    echo "tolearn запущен — закройте приложение и повторите" >&2
    exit 1
fi

echo "== сборка $product"
(cd "$root/app" && cargo tauri build "$@")

dmg=$(ls -t "$root/target/release/bundle/dmg/${product}_"*.dmg 2>/dev/null | head -1)
if [ -z "$dmg" ]; then
    echo "сборка прошла, а .dmg нет: пусто в target/release/bundle/dmg" >&2
    exit 1
fi

mount=$(mktemp -d "${TMPDIR:-/tmp}/tolearn-install-XXXXXX")
attached=0
cleanup() {
    if [ "$attached" = 1 ]; then
        hdiutil detach "$mount" -quiet >/dev/null 2>&1 || true
    fi
    rmdir "$mount" >/dev/null 2>&1 || true
}
trap cleanup EXIT INT TERM

echo "== $(basename "$dmg")"
hdiutil attach "$dmg" -nobrowse -readonly -quiet -mountpoint "$mount"
attached=1

if [ ! -d "$mount/$product.app" ]; then
    echo "в образе нет $product.app" >&2
    exit 1
fi

target="$destination/$product.app"
case "$target" in
    *.app) ;;
    *)
        echo "путь установки не заканчивается на .app: $target" >&2
        exit 1
        ;;
esac

mkdir -p "$destination"
if [ -e "$target" ]; then
    echo "== замена $target"
    rm -rf "$target"
fi
ditto "$mount/$product.app" "$target"

echo "установлено: $target"
