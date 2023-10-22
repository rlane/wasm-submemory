#!/bin/bash -eux
cd $(realpath $(dirname $0))

for SRC in *.rs; do
        DST="${SRC%.rs}.wasm"
        rustc --crate-name testcrate --edition=2021 --crate-type=cdylib -o "$DST" \
          --target=wasm32-unknown-unknown -C opt-level=s -C link-arg=-zstack-size=16384 "$SRC"
done

for SRC in *.c; do
        DST="${SRC%.c}.wasm"
        clang --target=wasm32-unknown-unknown -Oz \
                -nostdlib \
                -Wl,--export-all \
                -Wl,--no-entry \
                "$SRC" -o "$DST"
done

for SRC in *.zig; do
        DST="${SRC%.zig}.wasm"
        zig build-lib -O ReleaseSmall -target wasm32-freestanding --export=entry -dynamic \
                --export-memory --initial-memory=65536 --stack 16384 \
                "$SRC"
        rm "${DST}.o"
done