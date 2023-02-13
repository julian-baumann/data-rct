#!/usr/bin/env zsh

FFI_PROJECT="../data_rct_ffi/Cargo.toml"

# Colors
CYAN="\e[36m"
GREEN="\e[32m"
ENDCOLOR="\e[0m"

function PrintInfo()
{
    echo -e "${CYAN}$1${ENDCOLOR}"
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
    
    PrintDone
}

function GenerateUniffiBindings()
{
    PrintInfo "Generating bindings"
    pushd ../data_rct_ffi
        cargo run --bin uniffi-bindgen generate "src/data_rct.udl" --language swift --out-dir "../swift/Sources/DataRCT"
    popd

    mv Sources/DataRCT/*.h .out/headers/
    mv Sources/DataRCT/*.modulemap .out/headers/module.modulemap
    
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
          "../data_rct_ffi/target/$FirstArchitecture/release/libdata_rct_ffi.a" \
          -output ".out/$Target/libdata_rct_ffi.a"
    else
        lipo -create \
          "../data_rct_ffi/target/$FirstArchitecture/release/libdata_rct_ffi.a" \
          "../data_rct_ffi/target/$SecondArchitecture/release/libdata_rct_ffi.a" \
          -output ".out/$Target/libdata_rct_ffi.a"
    fi
    
    PrintDone
}

function GenerateXcFramework()
{
    PrintInfo "Generating xc-framework"

    
    rm -rf DataRCTFFI.xcframework

    xcodebuild -create-xcframework \
      -library ./.out/macos/libdata_rct_ffi.a \
      -headers .out/headers/ \
      -library ./.out/ios/libdata_rct_ffi.a \
      -headers .out/headers/ \
      -library ./.out/ios-simulator/libdata_rct_ffi.a \
      -headers .out/headers/ \
      -output DataRCTFFI.xcframework
      
      PrintDone
}



# ======= main =======

rm -rf .out
mkdir .out
mkdir .out/headers
mkdir .out/macos
mkdir .out/ios
mkdir .out/ios-simulator

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

rm -rf .out
