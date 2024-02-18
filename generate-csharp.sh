#!/bin/bash

pushd src/data_rct_ffi
    cargo build src/data_rct_ffi --lib --release --feautures sync --target x86_64-pc-windows-msvc
    cargo build src/data_rct_ffi --lib --release --feautures sync --target aarch64-pc-windows-msvc
popd

uniffi-bindgen-cs target/x86_64-pc-windows-msvc/release/libdata_rct_ffi.so --library --out-dir="bindings/csharp/DataRct/"
