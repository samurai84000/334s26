use serde::{Serialize, Deserialize};
// ring::signature::{self} allows using signature::ED25519 or importing it directly
use ring::signature::{self, Ed25519KeyPair, Signature, KeyPair, UnparsedPublicKey, ED25519};
use std::collections::HashMap;
use crate::crypto::hash::{Hashable, H256};


#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct RawTransaction {
    pub data: String,
    pub amount: u64,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {
    pub raw: RawTransaction,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
}

// Fixed: Implemented Hashable for Transaction as required by block.rs
impl Hashable for Transaction {
    fn hash(&self) -> H256 {
        let bytes = bincode::serialize(&self).unwrap();
        ring::digest::digest(&ring::digest::SHA256, &bytes).into()
    }
}

pub fn sign(t: &RawTransaction, key: &Ed25519KeyPair) -> Signature {
    let msg = bincode::serialize(t).expect("Serialization failed");
    key.sign(&msg)
}
/// Verify digital signature of a transaction, using public key instead of secret key
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