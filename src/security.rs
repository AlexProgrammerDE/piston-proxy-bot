use ed25519_compact::{PublicKey, Signature};

const PUBLIC_KEY_LENGTH: usize = 32;
const SIGNATURE_LENGTH: usize = 64;

pub fn verify_discord_request(
    public_key: &str,
    signature: &str,
    timestamp: &str,
    body: &[u8],
) -> bool {
    let mut public_key_bytes = [0; PUBLIC_KEY_LENGTH];
    let mut signature_bytes = [0; SIGNATURE_LENGTH];

    if hex::decode_to_slice(public_key, &mut public_key_bytes).is_err()
        || hex::decode_to_slice(signature, &mut signature_bytes).is_err()
    {
        return false;
    }

    let mut message = Vec::with_capacity(timestamp.len() + body.len());
    message.extend_from_slice(timestamp.as_bytes());
    message.extend_from_slice(body);

    PublicKey::new(public_key_bytes)
        .verify(&message, &Signature::new(signature_bytes))
        .is_ok()
}

#[cfg(test)]
mod tests {
    use ed25519_compact::{KeyPair, Seed};

    use super::verify_discord_request;

    #[test]
    fn verifies_the_timestamp_and_body_as_one_signed_message() {
        let key_pair = KeyPair::from_seed(Seed::new([42; 32]));
        let timestamp = "1720123456";
        let body = br#"{"type":1}"#;
        let message = [timestamp.as_bytes(), body].concat();
        let signature = key_pair.sk.sign(message, None);

        assert!(verify_discord_request(
            &hex::encode(key_pair.pk.as_ref()),
            &hex::encode(signature.as_ref()),
            timestamp,
            body,
        ));
        assert!(!verify_discord_request(
            &hex::encode(key_pair.pk.as_ref()),
            &hex::encode(signature.as_ref()),
            timestamp,
            br#"{"type":2}"#,
        ));
        assert!(!verify_discord_request(
            &hex::encode(key_pair.pk.as_ref()),
            &hex::encode(signature.as_ref()),
            "1720123457",
            body,
        ));
    }

    #[test]
    fn rejects_malformed_keys_and_signatures() {
        assert!(!verify_discord_request(
            "not-a-public-key",
            &"00".repeat(64),
            "1720123456",
            b"body",
        ));
        assert!(!verify_discord_request(
            &"00".repeat(32),
            "not-a-signature",
            "1720123456",
            b"body",
        ));
    }
}
