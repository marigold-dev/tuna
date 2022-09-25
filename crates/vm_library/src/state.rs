use fnv::FnvHashMap;
use serde::{Deserialize, Serialize};
use wasmer::Module;

use crate::{
    compile_store,
    contract_address::ContractAddress,
    errors::{vm::VmError, VMResult},
    outgoing::Init,
};
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContractType {
    pub self_: ContractAddress,
    pub originated_by: String,
    pub storage: Vec<u8>,
    #[serde(skip_deserializing, skip_serializing)]
    pub module: Option<Box<Module>>,
    pub serialized_module: Vec<u8>,
    pub constants: Vec<u8>,
}
impl ContractType {
    pub fn set_storage(&mut self, s: Vec<u8>) {
        self.storage = s
    }
    pub fn init(&mut self) -> VMResult<()> {
        match self.module {
            None => {
                self.module = Some(Box::from(unsafe {
                    Module::deserialize(&compile_store::new_headless(), &self.serialized_module)
                }?));
                Ok::<(), VmError>(())
            }
            Some(_) => Ok(()),
        }
    }
}
impl PartialEq for ContractType {
    fn eq(&self, other: &Self) -> bool {
        self.self_ == other.self_
    }
}
impl Eq for ContractType {
    fn assert_receiver_is_total_eq(&self) {}
}

#[derive(PartialEq, Eq, Debug)]
pub struct State {
    pub table: FnvHashMap<ContractAddress, ContractType>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            table: FnvHashMap::with_capacity_and_hasher(1000, Default::default()),
        }
    }
}

impl State {
    pub fn set(&mut self, key: ContractAddress, value: ContractType) -> Option<ContractType> {
        self.table.insert(key, value)
    }

    pub fn get(&mut self, key: &ContractAddress) -> Option<ContractType> {
        self.table.remove(key)
    }
    pub fn from_init(&mut self, init: Init) -> VMResult<()> {
        self.table.clear();
        init.0.iter().try_for_each(|(key, value)| {
            let contract_type: ContractType = bincode::deserialize_from(value.as_bytes())
                .map_err(|err| VmError::DeserializeErr(err.to_string()))?;
            self.table.insert(key.clone(), contract_type);
            Ok(())
        })
    }
    pub fn to_init(&self) -> VMResult<Init> {
        let mut acc = FnvHashMap::with_capacity_and_hasher(self.table.len(), Default::default());
        self.table
            .iter()
            .try_for_each(|(contract_address, contract_type)| {
                let immediate: ContractType = contract_type.clone();
                let immediate = bincode::serialize(&immediate)
                    .map_err(|err| VmError::DeserializeErr(err.to_string()))?;
                acc.insert(
                    contract_address.clone(),
                    String::from_utf8_lossy(&immediate).to_string(),
                );
                Ok::<(), VmError>(())
            })?;
        Ok(Init(acc))
    }
}
