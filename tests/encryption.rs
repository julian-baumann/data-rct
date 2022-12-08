use std::io::{Cursor, Read, Write};
use chacha20::cipher::{KeyIvInit, StreamCipher};
use chacha20::XChaCha20;
use rand_core::OsRng;
use x25519_dalek::{EphemeralSecret, PublicKey};
use data_rct::encryption::{EncryptedStream, generate_nonce};
use crate::helper::MemoryStream;

mod helper;

#[test]
pub fn diffie_hellman() {
    let alice_secret = EphemeralSecret::new(OsRng);
    let alice_public_key = PublicKey::from(&alice_secret);

    let bob_secret = EphemeralSecret::new(OsRng);
    let bob_public_key = PublicKey::from(&bob_secret);

    assert_ne!(alice_public_key.as_bytes(), bob_public_key.as_bytes());

    let alice_shared_secret = alice_secret.diffie_hellman(&bob_public_key);
    let bob_shared_secret = bob_secret.diffie_hellman(&alice_public_key);

    assert_eq!(alice_shared_secret.as_bytes(), bob_shared_secret.as_bytes());
}

#[test]
pub fn stream_encryption() {
    let key: [u8; 32] = [
        0x4b, 0x62, 0xe9, 0xd4, 0xd1, 0xb4, 0x67, 0x3c, 0x5a, 0xd2, 0x26, 0x91, 0x95, 0x7d,
        0x6a, 0xf5, 0xc3, 0x1b, 0x64, 0x21, 0xe0, 0xea, 0x01, 0xd4, 0x2c, 0xa4, 0x16, 0x9e,
        0x79, 0x18, 0xba, 0x1d,
    ];

    let nonce = generate_nonce();

    let mut memory_stream = MemoryStream::new();
    let mut encrypted_stream = EncryptedStream::new(&key, &nonce, Box::new(&mut memory_stream));

    let write_data = &vec![1, 2, 3];

    let written_bytes = encrypted_stream.write(write_data)
        .expect("Something went wrong, while trying to write to EncryptedStream");

    assert!(written_bytes > 0);

    // let mut encrypted_gibberish = Vec::new();
    // encrypted_stream.raw_stream.read_to_end(&mut encrypted_gibberish)
    //     .expect("Error reading memory_stream");
    //
    // assert_ne!(write_data, &encrypted_gibberish);
    //
    // let written_bytes = encrypted_stream.write(write_data)
    //     .expect("Something went wrong, while trying to write to EncryptedStream");
    //
    // assert!(written_bytes > 0);

    let mut decrypted_buffer = [0u8, 19];
    encrypted_stream.read_last(&mut decrypted_buffer)
        .expect("Error reading memory_stream");

    assert_eq!(write_data, &decrypted_buffer);
}