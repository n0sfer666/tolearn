#!/bin/sh
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

list=${TMPDIR:-/tmp}/tolearn-checks-$$
trap 'rm -f "$list"' EXIT INT TERM

node -e '
const listed = require("./.context/checks.json");
const rows = Object.entries(listed).map(([name, command]) => name + "\t" + command);
process.stdout.write(rows.join("\n") + "\n");
' >"$list"

for place in ui obsidian; do
    if [ ! -d "$place/node_modules" ]; then
        echo "== зависимости $place"
        pnpm -C "$place" install --frozen-lockfile
    fi
done

if [ ! -d ui/dist ]; then
    echo "== сборка интерфейса (его вшивает в себя приложение)"
    pnpm -C ui build
fi

total=$(wc -l <"$list" | tr -d ' ')
step=0

while IFS='	' read -r name command; do
    step=$((step + 1))
    printf '== [%s/%s] %s: %s\n' "$step" "$total" "$name" "$command"
    if ! sh -c "$command"; then
        printf 'провалено на %s: %s\n' "$name" "$command" >&2
        exit 1
    fi
done <"$list"

printf 'все проверки пройдены: %s из %s\n' "$total" "$total"
