use blake2::digest::consts::U20;
use blake2::Digest;
type Blake2b160 = blake2::Blake2b<U20>;
use serde::{Deserialize, Serialize};
use sha2::{Digest as D, Sha256};

fn checksum(mut s: Vec<u8>) -> Vec<u8> {
    let one = Sha256::digest(&s);
    s.append(&mut Sha256::digest(&one)[0..4].to_vec());
    s
}

fn encode(s: &[u8]) -> String {
    let mut prefix = vec![1, 146, 6];
    prefix.append(&mut s.to_vec());
    bs58::encode(checksum(prefix))
        .with_alphabet(bs58::Alphabet::BITCOIN)
        .into_string()
}
pub fn decode(s: &[u8]) -> Result<Vec<u8>, bs58::decode::Error> {
    let s = bs58::decode(&s)
        .with_alphabet(bs58::Alphabet::BITCOIN)
        .into_vec()?;
    let s = &s[3..s.len() - 4];
    Ok(s.to_vec())
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Hash, PartialOrd, Ord)]
pub struct ContractAddress(pub String);
impl ContractAddress {
    pub fn new(s: &[u8]) -> Self {
        Self(encode(&Blake2b160::digest(s)))
    }
}
