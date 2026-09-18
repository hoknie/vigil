#!/bin/sh
set -eu

cd "$(dirname "$0")/../.."

into="${1:-target/vigil-config}"
case "$into" in
/*) ;;
*) into="$PWD/$into" ;;
esac

rm -rf "$into"
mkdir -p "$into/collectors" "$into/suppressions" "$into/reporters"
chmod 0700 "$into" "$into/collectors" "$into/suppressions" "$into/reporters"

placed() {
    sed "s#/etc/vigil/#$into/#g" "$1" > "$2"
    chmod 0600 "$2"
}

placed config/vigil.example.yaml "$into/vigil.yaml"
placed config/watch_fs.yaml "$into/watch_fs.yaml"
for shipped in config/collectors/*.yaml; do
    placed "$shipped" "$into/collectors/$(basename "$shipped")"
done

if [ -n "${SCHEDULE:-}" ]; then
    for block in "$into"/collectors/*.yaml; do
        sed "s/^  schedule: .*/  schedule: $SCHEDULE/" "$block" > "$block.writing"
        mv "$block.writing" "$block"
        chmod 0600 "$block"
    done
fi

echo "$into/vigil.yaml"
