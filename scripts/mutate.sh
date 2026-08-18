#!/bin/sh
set -eu

near=${MUTATE_NEAR_DIR:-target/near}
far=${MUTATE_FAR_DIR:-target/far}
near_limit=${MUTATE_NEAR_LIMIT:-900}
far_limit=${MUTATE_FAR_LIMIT:-2700}
whole_limit=${MUTATE_LIMIT:-10800}
log=${MUTATE_LOG:-target/mutate.log}
began=$(date +%s)

note() { printf '%s\n' "$*" >>"$log"; }

brood() {
    for child in $(pgrep -P "$1" 2>/dev/null || true); do
        brood "$child"
        echo "$child"
    done
}

felled() {
    for pid in $(brood "$1") "$1"; do
        kill -TERM "$pid" 2>/dev/null || true
    done
    sleep 2
    for pid in $(brood "$1") "$1"; do
        kill -KILL "$pid" 2>/dev/null || true
    done
}

bounded() {
    limit=$1
    shift
    "$@" >>"$log" 2>&1 </dev/null &
    child=$!
    spent=0
    while kill -0 "$child" 2>/dev/null && [ "$spent" -lt "$limit" ]; do
        sleep 1
        spent=$((spent + 1))
    done
    if kill -0 "$child" 2>/dev/null; then
        felled "$child"
        wait "$child" 2>/dev/null || true
        return 124
    fi
    code=0
    wait "$child" || code=$?
    return "$code"
}

circle() {
    note "== близкий круг $(date +%H:%M:%S)"
    bounded "$near_limit" env CARGO_TARGET_DIR="$near" \
        cargo nextest run --workspace --exclude tolearn-app --cargo-profile mutants
}

whole() {
    note "== дальний круг $(date +%H:%M:%S)"
    bounded "$far_limit" env CARGO_TARGET_DIR="$far" \
        cargo nextest run --workspace --no-fail-fast --cargo-profile mutants
}

said() {
    case "$1" in
        100) echo "$2" ;;
        101) echo "не-собралась" ;;
        124) echo "таймаут-$3" ;;
        *) echo "ошибка-раннера-$1" ;;
    esac
}

verdict() {
    code=0
    circle || code=$?
    if [ "$code" -ne 0 ]; then said "$code" убита близкий; return 0; fi
    code=0
    whole || code=$?
    if [ "$code" -ne 0 ]; then said "$code" убита-дальним дальний; return 0; fi
    echo "ВЫЖИЛА"
}

baseline() {
    code=0
    circle || code=$?
    if [ "$code" -eq 0 ]; then return 0; fi
    echo "чистое дерево не зелёное ($(said "$code" "тесты-падают" близкий)) — вердикты были бы ложными, смотри $log" >&2
    exit 1
}

applied() {
    WAS=$2 NOW=$3 perl -0777 -i -pe 's/\Q$ENV{WAS}\E/$ENV{NOW}/' "$1"
    ! git diff --quiet -- "$1"
}

one() {
    step=$(date +%s)
    told=$(verdict)
    printf '%s\t%s\t%sс\n' "$1" "$told" "$(($(date +%s) - step))"
}

clean() {
    git diff --quiet -- "$1" && git diff --cached --quiet -- "$1"
}

batch() {
    baseline
    total=$(awk 'NF && $0 !~ /^[[:space:]]*#/' "$1" | wc -l | tr -d ' ')
    done_=0
    tab=$(printf '\t')
    while IFS="$tab" read -r name file was now; do
        case "${name:-#}" in '#'*|'') continue ;; esac
        done_=$((done_ + 1))
        if [ $(($(date +%s) - began)) -ge "$whole_limit" ]; then
            printf '%s/%s\t%s\tпропущена-общий-таймаут\n' "$done_" "$total" "$name"
            continue
        fi
        if ! clean "$file"; then
            printf '%s/%s\t%s\tфайл-грязный\n' "$done_" "$total" "$name"
            continue
        fi
        if applied "$file" "$was" "$now"; then
            printf '%s/%s\t%s\n' "$done_" "$total" "$(one "$name")"
        else
            git checkout -- "$file"
            printf '%s/%s\t%s\tне-наложилась\n' "$done_" "$total" "$name"
        fi
        git checkout -- "$file"
    done <"$1"
}

mkdir -p "$(dirname "$log")"
: >"$log"

case "${1:-}" in
    -l) batch "${2:?файл набора мутаций}" ;;
    '') one "дерево" ;;
    *) echo "usage: mutate.sh [-l набор.tsv]" >&2; exit 1 ;;
esac

echo "лог: $log"
