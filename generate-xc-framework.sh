#!/usr/bin/env zsh

FFI_PROJECT="src/data_rct_ffi/Cargo.toml"

# Colors
CYAN="\e[36m"
RED="\e[0;31m"
GREEN="\e[32m"
ENDCOLOR="\e[0m"

function PrintInfo()
{
    echo -e "${CYAN}$1${ENDCOLOR}"
}

function CheckForErrorAndExitIfNecessary()
{
    if [ "$?" != "0" ]; then echo -e "${RED}$1${ENDCOLOR}"; exit 1; fi
}

function PrintDone()
{
    echo -e "    ${GREEN}Done${ENDCOLOR}"
    echo ""
    echo ""
}

function BuildStaticLibrary()
{
    Target=$1
    PrintInfo "Building for $Target"
    cargo build --manifest-path $FFI_PROJECT --lib --release --target $Target
    CheckForErrorAndExitIfNecessary

    PrintDone
}

function GenerateUniffiBindings()
{
    PrintInfo "Generating bindings"
    cargo run --bin uniffi-bindgen generate "src/data_rct_ffi/src/data_rct.udl" --language swift --out-dir "bindings/swift/Sources/DataRCT"
    CheckForErrorAndExitIfNecessary

    pushd bindings/swift
        mv Sources/DataRCT/*.h .out/headers/
        mv Sources/DataRCT/*.modulemap .out/headers/module.modulemap
    popd

    PrintDone
}

function CreateUniversalBinary()
{
    Target=$1
    FirstArchitecture=$2
    SecondArchitecture=$3

    PrintInfo "Generating universal binary for $Target"

    if [ -z "$SecondArchitecture" ]
    then
        lipo -create \
          "target/$FirstArchitecture/release/libdata_rct_ffi.a" \
          -output "bindings/swift/.out/$Target/libdata_rct_ffi.a"

        CheckForErrorAndExitIfNecessary
    else
        lipo -create \
          "target/$FirstArchitecture/release/libdata_rct_ffi.a" \
          "target/$SecondArchitecture/release/libdata_rct_ffi.a" \
          -output "bindings/swift/.out/$Target/libdata_rct_ffi.a"

        CheckForErrorAndExitIfNecessary
    fi

    PrintDone
}

function GenerateXcFramework()
{
    PrintInfo "Generating xc-framework"

    rm -rf bindings/swift/DataRCTFFI.xcframework

    xcodebuild -create-xcframework \
      -library bindings/swift/.out/macos/libdata_rct_ffi.a \
      -headers bindings/swift/.out/headers/ \
      -library bindings/swift/.out/ios/libdata_rct_ffi.a \
      -headers bindings/swift/.out/headers/ \
      -library bindings/swift/.out/ios-simulator/libdata_rct_ffi.a \
      -headers bindings/swift/.out/headers/ \
      -output bindings/swift/DataRCTFFI.xcframework

    CheckForErrorAndExitIfNecessary
    PrintDone
}



# ======= main =======

rm -rf bindings/swift/.out
mkdir bindings/swift/.out
mkdir bindings/swift/.out/headers
mkdir bindings/swift/.out/macos
mkdir bindings/swift/.out/ios
mkdir bindings/swift/.out/ios-simulator

# iOS
BuildStaticLibrary aarch64-apple-ios

# iOS Simulator
BuildStaticLibrary aarch64-apple-ios-sim
BuildStaticLibrary x86_64-apple-ios

# macOS
BuildStaticLibrary x86_64-apple-darwin
BuildStaticLibrary aarch64-apple-darwin

GenerateUniffiBindings

CreateUniversalBinary "macos" "x86_64-apple-darwin" "aarch64-apple-darwin"
CreateUniversalBinary "ios" "aarch64-apple-ios"
CreateUniversalBinary "ios-simulator" "x86_64-apple-ios" "aarch64-apple-ios-sim"

GenerateXcFramework

#zip -r DataRCTFFI.xcframework.zip DataRCTFFI.xcframework

rm -rf bindings/swift/.out
