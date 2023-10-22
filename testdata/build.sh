#!/bin/bash -eux
cd $(realpath $(dirname $0))
mkdir -p wasm/rust wasm/c wasm/zig

for SRC in rust/*.rs; do
        DST=wasm/"${SRC%.rs}.wasm"
        rustc --crate-name testcrate --edition=2021 --crate-type=cdylib -o "$DST" \
          --target=wasm32-unknown-unknown -C opt-level=s -C link-arg=-zstack-size=16384 "$SRC"
done

for SRC in c/*.c; do
        DST=wasm/"${SRC%.c}.wasm"
        clang --target=wasm32-unknown-unknown -Oz \
                -nostdlib \
                -Wl,--export-all \
                -Wl,--no-entry \
                "$SRC" -o "$DST"
done

for SRC in zig/*.zig; do
        DST=wasm/"${SRC%.zig}.wasm"
        zig build-lib -O ReleaseSmall -target wasm32-freestanding --export=entry -dynamic \
                --export-memory --initial-memory=65536 --stack 16384 \
                "$SRC"
        mv *.wasm "$DST"
        rm *.wasm.o
done
