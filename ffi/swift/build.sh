#!/usr/bin/env bash

pushd ../data_rct_ffi 
    cargo run --bin uniffi-bindgen generate "src/data_rct.udl" --language swift --out-dir "../swift/Sources/DataRCT"
popd

rm -rf Include
mkdir Include

mv Sources/DataRCT/*.h Include/
mv Sources/DataRCT/*.modulemap Include/module.modulemap

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
mkdir .out/macos
mkdir .out/ios
mkdir .out/ios-simulator


echo ""
echo "Generating dynamic macOS library"

# Generate dynamic macOS library
lipo -create \
  ../../target/x86_64-apple-darwin/release/libdata_rct.a \
  ../../target/aarch64-apple-darwin/release/libdata_rct.a \
  -output .out/macos/libdatarct.a

echo "Done."

echo ""
echo "Generating dynamic iOS library"

# Generate dynamic iOS library
lipo -create \
  ../../target/aarch64-apple-ios/release/libdata_rct.a \
  -output .out/ios/libdatarct.a

echo "Done."

echo ""
echo "Generating dynamic iOS simulator library"

# Generate dynamic iOS simulator library
lipo -create \
  ../../target/x86_64-apple-ios/release/libdata_rct.a \
  ../../target/aarch64-apple-ios-sim/release/libdata_rct.a \
  -output .out/ios-simulator/libdatarct.a

echo "Done."

echo ""
echo "Generating xcframework"

rm -rf DataRCTFFI.xcframework

xcodebuild -create-xcframework \
  -library ./.out/macos/libdatarct.a \
  -headers ./Include/ \
  -library ./.out/ios/libdatarct.a \
  -headers ./Include/ \
  -library ./.out/ios-simulator/libdatarct.a \
  -headers ./Include/ \
  -output DataRCTFFI.xcframework

#zip -r DataRCTFFI.xcframework.zip DataRCTFFI.xcframework

#rm -rf .out

echo "Done."
