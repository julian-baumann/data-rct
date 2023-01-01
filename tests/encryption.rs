use std::io::{Cursor, Read, Write};
use rand_core::{OsRng, RngCore};
use x25519_dalek::{EphemeralSecret, PublicKey};
use data_rct::encryption::{EncryptedStream, generate_key, generate_nonce};
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
    let key = generate_key();
    let nonce = generate_nonce();

    let mut memory_stream = MemoryStream::new();
    let mut encrypted_stream = EncryptedStream::new(key.as_slice(), nonce.as_slice(), Box::new(&mut memory_stream));

    let write_data = &vec![1, 2, 3];

    let written_bytes = encrypted_stream.write(write_data)
        .expect("Something went wrong, while trying to write to EncryptedStream");

    assert!(written_bytes > 0);

    let mut encrypted_gibberish = Vec::new();
    encrypted_stream.raw_stream.read_to_end(&mut encrypted_gibberish)
        .expect("Error reading memory_stream");

    assert_ne!(write_data, &encrypted_gibberish);


    // =========

    let written_bytes = encrypted_stream.write(write_data)
        .expect("Something went wrong, while trying to write to EncryptedStream");

    assert!(written_bytes > 0);

    let mut decrypted_buffer = Vec::new();
    let read_bytes = encrypted_stream.read_to_end(&mut decrypted_buffer)
        .expect("Error reading memory_stream");

    assert_eq!(write_data, &decrypted_buffer[..read_bytes]);
}


#[test]
pub fn large_stream_encryption() {
    let key = generate_key();
    let nonce = generate_nonce();

    let mut memory_stream = MemoryStream::new();
    let mut encrypted_stream = EncryptedStream::new(key.as_slice(), nonce.as_slice(), Box::new(&mut memory_stream));


    let mut write_data: [u8; 100] = [0; 100];
    let rng = &mut OsRng;
    rng.fill_bytes(&mut write_data);

    let write_data = write_data.as_slice();


    let written_bytes = encrypted_stream.write(write_data)
        .expect("Something went wrong, while trying to write to EncryptedStream");

    assert!(written_bytes > 0);

    let mut encrypted_gibberish = Vec::new();
    encrypted_stream.raw_stream.read_to_end(&mut encrypted_gibberish)
        .expect("Error reading memory_stream");

    assert_ne!(write_data, &encrypted_gibberish);


    // =========

    let written_bytes = encrypted_stream.write(write_data)
        .expect("Something went wrong, while trying to write to EncryptedStream");

    assert!(written_bytes > 0);

    let mut decrypted_buffer: [u8; 100] = [0; 100];
    let read_bytes = encrypted_stream.read_last(&mut decrypted_buffer)
        .expect("Error reading memory_stream");

    assert_eq!(write_data, &decrypted_buffer[..read_bytes]);
}