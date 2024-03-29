# Navigate to the src/data_rct_ffi directory
Push-Location src/data_rct_ffi

# Build for x86_64-pc-windows-msvc
cargo build --lib --release --features sync --target x86_64-pc-windows-msvc
# Build for aarch64-pc-windows-msvc
# cargo build --lib --release --features sync --target aarch64-pc-windows-msvc

# Return to the previous directory
Pop-Location

# Run uniffi-bindgen for C# bindings generation
& uniffi-bindgen-cs.exe target/x86_64-pc-windows-msvc/release/data_rct_ffi.dll --library --out-dir="bindings/csharp/DataRct/"
