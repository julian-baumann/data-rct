# DataRCT

[![CI](https://github.com/julian-baumann/data-rct/actions/workflows/ci.yml/badge.svg)](https://github.com/julian-baumann/data-rct/actions/workflows/ci.yml)
![](https://www.repostatus.org/badges/latest/wip.svg)

DataRCT is a new protocol with the goal of establishing a secure and reliable connection between two nearby devices by using common technologies like TCP or BLE.

This protocol is designed to be used without the need to know which transport medium is used for transmission. DataRCT always uses the fastest possible connection.

## Encryption

The network stream is encrypted using the `XChaCha20` algorithm ([using this crate](https://crates.io/crates/chacha20)).