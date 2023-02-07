#!/usr/bin/env bash

pushd ../data_rct_ffi 
    cargo run --bin uniffi-bindgen generate "src/data_rct.udl" --language swift --out-dir "../swift/Sources/DataRCT"
popd

rm -rf Include
mkdir Include

mv Sources/DataRCT/*.h Include/
mv Sources/DataRCT/*.modulemap Include/

Configuration="Release"

# iOS
ARCHS="arm64" ./xc-universal-binary.sh data_rct ../data_rct_ffi $Configuration

# macOS
IS_MAC=1 ARCHS="x86_64" ./xc-universal-binary.sh data_rct ../data_rct_ffi $Configuration
IS_MAC=1 ARCHS="arm64" ./xc-universal-binary.sh data_rct ../data_rct_ffi $Configuration

# iOS Simulator
LLVM_TARGET_TRIPLE_SUFFIX="-simulator" ARCHS="arm64" ./xc-universal-binary.sh data_rct ../data_rct_ffi $Configuration
LLVM_TARGET_TRIPLE_SUFFIX="-simulator" ARCHS="x86_64" ./xc-universal-binary.sh data_rct ../data_rct_ffi $Configuration

rm -rf .out
mkdir .out

echo ""
echo "Generating dynamic macOS library"

# Generate dynamic macOS library
lipo -create \
  ../../target/x86_64-apple-darwin/release/libdata_rct.a \
  ../../target/aarch64-apple-darwin/release/libdata_rct.a \
  -output .out/libdatarct_macos.a

echo "Done."

echo ""
echo "Generating dynamic iOS library"

# Generate dynamic iOS library
lipo -create \
  ../../target/aarch64-apple-ios/release/libdata_rct.a \
  -output .out/libdatarct_ios.a

echo "Done."

echo ""
echo "Generating dynamic iOS simulator library"

# Generate dynamic iOS simulator library
lipo -create \
  ../../target/x86_64-apple-ios/release/libdata_rct.a \
  ../../target/aarch64-apple-ios-sim/release/libdata_rct.a \
  -output .out/libdatarct_ios_simulator.a

echo "Done."

echo ""
echo "Generating xcframework"

xcodebuild -create-xcframework \
  -library ./.out/libdatarct_macos.a \
  -headers ./Include/ \
  -library ./.out/libdatarct_ios.a \
  -headers ./Include/ \
  -library ./.out/libdatarct_ios_simulator.a \
  -headers ./Include/ \
  -output .out/DataRCT_FFI.xcframework

zip -r DataRCT_FFI.xcframework.zip .out/DataRCT_FFI.xcframework

rm -rf .out

echo "Done."
