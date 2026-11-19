//! Signs a Discord interaction with a fuzzer-chosen key, checks that the
//! Worker accepts the genuine signature, then applies one mutation and checks
//! that every malformed, truncated or tampered request is rejected.
//!
//! Mutations that leave the request equivalent are skipped: hex is
//! case-insensitive, and Discord signs `timestamp || body`, so moving bytes
//! between the timestamp and the body keeps the signed message the same.

#![no_main]

use arbitrary::Arbitrary;
use ed25519_compact::{KeyPair, Seed};
use libfuzzer_sys::fuzz_target;
use piston_proxy_bot::fuzzing::verify_discord_request;

#[derive(Arbitrary, Debug)]
struct Input {
    seed: [u8; 32],
    timestamp: String,
    body: Vec<u8>,
    mutation: Mutation,
}

#[derive(Arbitrary, Debug)]
enum Mutation {
    TruncateSignature(u8),
    ExtendSignature(String),
    ReplaceSignatureChar { index: u8, replacement: char },
    FlipSignatureBit(u16),
    ReplaceSignature(String),
    ReplaceTimestamp(String),
    ReplaceBody(Vec<u8>),
    ReplacePublicKey(String),
    UseOtherKey([u8; 32]),
}

fuzz_target!(|input: Input| {
    let Input {
        seed,
        timestamp,
        body,
        mutation,
    } = input;
    // `ed25519-compact` refuses all-zero seeds.
    let Ok(key_pair) = KeyPair::try_from_seed(Seed::new(seed)) else {
        return;
    };
    let public_key = hex::encode(key_pair.pk.as_ref());
    let message = [timestamp.as_bytes(), &body].concat();
    let signature = hex::encode(key_pair.sk.sign(&message, None).as_ref());

    assert!(
        verify_discord_request(&public_key, &signature, &timestamp, &body),
        "a genuine signature was rejected"
    );

    let accepted = match mutation {
        Mutation::TruncateSignature(length) => {
            let length = usize::from(length) % signature.len();
            verify_discord_request(&public_key, &signature[..length], &timestamp, &body)
        }
        Mutation::ExtendSignature(suffix) => {
            if suffix.is_empty() {
                return;
            }
            let extended = format!("{signature}{suffix}");
            verify_discord_request(&public_key, &extended, &timestamp, &body)
        }
        Mutation::ReplaceSignatureChar { index, replacement } => {
            let index = usize::from(index) % signature.len();
            let original = char::from(signature.as_bytes()[index]);
            if replacement.eq_ignore_ascii_case(&original) {
                return;
            }
            let mut mutated = signature.clone();
            mutated.replace_range(index..=index, replacement.encode_utf8(&mut [0; 4]));
            verify_discord_request(&public_key, &mutated, &timestamp, &body)
        }
        Mutation::FlipSignatureBit(bit) => {
            let bit = usize::from(bit) % (signature.len() * 4);
            let mut bytes = hex::decode(&signature).expect("the signature is valid hex");
            bytes[bit / 8] ^= 1 << (bit % 8);
            verify_discord_request(&public_key, &hex::encode(bytes), &timestamp, &body)
        }
        Mutation::ReplaceSignature(replacement) => {
            if replacement.eq_ignore_ascii_case(&signature) {
                return;
            }
            verify_discord_request(&public_key, &replacement, &timestamp, &body)
        }
        Mutation::ReplaceTimestamp(replacement) => {
            if [replacement.as_bytes(), &body].concat() == message {
                return;
            }
            verify_discord_request(&public_key, &signature, &replacement, &body)
        }
        Mutation::ReplaceBody(replacement) => {
            if [timestamp.as_bytes(), &replacement].concat() == message {
                return;
            }
            verify_discord_request(&public_key, &signature, &timestamp, &replacement)
        }
        Mutation::ReplacePublicKey(replacement) => {
            if replacement.eq_ignore_ascii_case(&public_key) {
                return;
            }
            verify_discord_request(&replacement, &signature, &timestamp, &body)
        }
        Mutation::UseOtherKey(other_seed) => {
            let Ok(other_key_pair) = KeyPair::try_from_seed(Seed::new(other_seed)) else {
                return;
            };
            let other_key = hex::encode(other_key_pair.pk.as_ref());
            if other_key == public_key {
                return;
            }
            verify_discord_request(&other_key, &signature, &timestamp, &body)
        }
    };

    assert!(!accepted, "a mutated request was accepted");
});
