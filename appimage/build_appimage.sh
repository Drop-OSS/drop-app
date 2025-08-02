#!/bin/sh

cd ../
git submodule init
git submodule update --recursive

yarn
yarn tauri build --no-bundle

cp src-tauri/target/release/drop-app appimage/drop-oss-app.d/usr/bin

# next thing is dependency fetching. how do we do this without tauri
