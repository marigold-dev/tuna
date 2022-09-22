use fnv::FnvHashMap;
use serde::{Deserialize, Serialize};

use crate::contract_address::ContractAddress;

#[derive(Serialize)]
pub struct Set<'a> {
    pub key: &'a ContractAddress,
    pub value: &'a str,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SetOwned {
    pub key: ContractAddress,
    pub value: String,
}

#[repr(transparent)]
#[derive(Deserialize, Serialize, Debug)]
pub struct Init(pub FnvHashMap<ContractAddress, String>);
#[repr(transparent)]
#[derive(Deserialize, Serialize, Debug)]
pub struct InitVec(pub Vec<SetOwned>);
