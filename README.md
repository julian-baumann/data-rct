# DataRCT

[![CI](https://github.com/julian-baumann/data-rct/actions/workflows/ci.yml/badge.svg)](https://github.com/julian-baumann/data-rct/actions/workflows/ci.yml)
![](https://www.repostatus.org/badges/latest/wip.svg)

DataRCT is a new protocol with the goal of establishing a secure and reliable connection between two nearby devices by using common technologies like TCP or BLE.

This protocol is designed so that it can be used without having to know which transport medium is used for the transmission. DataRCT always uses the fastest connection possible, by choosing the right stack out of possible implementations, like TCP or BLE.

## Encryption

> I am by no means a security expert. I did my best to secure the stream. If you may find any vulnerabilities or attack surfaces, let me know

The network stream is encrypted and authorized using the `XChaCha20Poly1305` algorithm ([using this crate](https://docs.rs/chacha20poly1305/0.10.1/chacha20poly1305/)).


## Progress

✅ = Done and published <br />
⏳ = Working on it  <br />
🗓 = Planned <br />

| Goal | State |
| --- | ----------- |
| UDP Discovery | ✅ |
| mDNS-SD Discovery | ✅ |
| BLE Discovery | 🗓 |
| TCP Transmission | ✅ |
| BLE Transmission | 🗓 |
| Stream encryption | ⏳ |
| Authorization | 🗓 |
| FFI Bindings for Swift | 🗓 |
| FFI Bindings for Kotlin | 🗓 |
| FFI Bindings for C# | 🗓 |
