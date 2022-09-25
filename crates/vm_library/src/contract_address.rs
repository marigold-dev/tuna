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
fn decode(s: String) -> Result<Vec<u8>, bs58::decode::Error> {
    let s = bs58::decode(&s)
        .with_alphabet(bs58::Alphabet::BITCOIN)
        .into_vec()?;
    let s = &s[3..s.len() - 4];
    Ok(s.to_vec())
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Hash, PartialOrd, Ord)]
pub struct ContractAddress(
    #[serde(serialize_with = "serializer", deserialize_with = "deserialize")] Vec<u8>,
);
impl ContractAddress {
    pub fn new(s: String) -> Self {
        Self(Blake2b160::digest(s).to_vec())
    }
}
impl ContractAddress {
    pub fn from_operation(s: String) -> Self {
        Self(Blake2b160::digest(s).to_vec())
    }
}

fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let s: String = serde::de::Deserialize::deserialize(deserializer)?;
    decode(s).map_err(|_| serde::de::Error::custom("error deserializing contract_address"))
}

fn serializer<D>(t: &[u8], serializer: D) -> Result<D::Ok, D::Error>
where
    D: serde::ser::Serializer,
{
    serializer.serialize_str(&encode(t))
}
