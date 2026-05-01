use serde::{Serialize, Deserialize};
// ring::signature::{self} allows using signature::ED25519 or importing it directly
use ring::signature::{self, Ed25519KeyPair, Signature, KeyPair, UnparsedPublicKey, ED25519};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct RawTransaction {
    pub data: String,
    pub amount: u64,
}

pub fn sign(t: &RawTransaction, key: &Ed25519KeyPair) -> Signature {
    let msg = bincode::serialize(t).expect("Serialization failed");
    key.sign(&msg) // No semicolon: returns Signature
}

pub fn verify(t: &RawTransaction, public_key: &[u8], signature: &[u8]) -> bool {
    let msg = bincode::serialize(t).expect("Serialization failed");
    let unparsed_key = UnparsedPublicKey::new(&ED25519, public_key);
    unparsed_key.verify(&msg, signature).is_ok()
}

#[cfg(any(test, test_utilities))]
mod tests {
    use super::*;
    use crate::crypto::key_pair; 

    pub fn generate_random_transaction() -> RawTransaction {
        RawTransaction {
            data: format!("Transaction-{}", rand::random::<u16>()),
            amount: rand::random::<u64>(),
        }
    }

    #[test]
    fn sign_verify() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        // Ensure we pass the public key and signature as byte slices
        assert!(verify(&t, key.public_key().as_ref(), signature.as_ref()));
    }
}