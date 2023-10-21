#!/bin/bash -eux
test -f Cargo.toml
cd wasm
for SRC in *.rs; do
        DST="${SRC%.rs}.wasm"
        rustc --crate-name testcrate --edition=2021 --crate-type=cdylib -o "$DST" \
          --target=wasm32-unknown-unknown -C opt-level=s -C link-arg=-zstack-size=16384 "$SRC"
done
