#!/bin/bash

function setupWindows() {
    echo "Setting up Windows target\n"
    rustup target add \
        x86_64-pc-windows-msvc \
        aarch64-pc-windows-msvc
    cargo install uniffi-bindgen-cs --git https://github.com/NordSecurity/uniffi-bindgen-cs --tag v0.8.0+v0.25.0
    dotnet tool install -g csharpier
}

function setupApple() {
    echo "Setting up iOS and macOS targets\n"

    rustup target add \
        aarch64-apple-ios \
        aarch64-apple-ios-sim \
        x86_64-apple-ios \
        x86_64-apple-darwin \
        aarch64-apple-ios-darwin
}

function setupAndroid() {
    echo "Setting up Android targets\n"

    rustup target add x86_64-linux-android \
        x86_64-unknown-linux-gnu \
        aarch64-linux-android \
        armv7-linux-androideabi \
        i686-linux-android
}

setupApple
setupAndroid
setupWindows
