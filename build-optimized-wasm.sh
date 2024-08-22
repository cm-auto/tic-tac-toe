#!/usr/bin/env bash

cargo build --release --target wasm32-unknown-unknown && cp target/wasm32-unknown-unknown/release/game.wasm . && wasm-opt game.wasm -o game.wasm -O --intrinsic-lowering -O