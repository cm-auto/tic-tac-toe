#!/usr/bin/env bash

watchexec -w src "cargo build --target wasm32-unknown-unknown && cp target/wasm32-unknown-unknown/debug/game.wasm ."