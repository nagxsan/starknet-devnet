use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use starknet_in_rust::services::api::contract_classes::deprecated_contract_class::ContractClass as StarknetInRustContractClass;
use starknet_rs_core::types::{CompressedLegacyContractClass, LegacyEntryPointsByType};

use crate::contract_class::deprecated::abi_entry::{AbiEntry, AbiEntryType};
use crate::contract_class::Cairo0Json;
use crate::error::{Error, JsonError};
use crate::felt::Felt;
use crate::serde_helpers::base_64_gzipped_json_string::{
    deserialize_to_serde_json_value_with_keys_ordered_in_alphabetical_order,
    serialize_program_to_base64,
};
use crate::traits::HashProducer;
use crate::DevnetResult;

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct ContractClassAbiEntryWithType {
    #[serde(flatten)]
    pub entry: AbiEntry,
    pub r#type: AbiEntryType,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeprecatedContractClass {
    /// A base64 encoding of the gzip-compressed JSON representation of program.
    #[serde(
        deserialize_with = "deserialize_to_serde_json_value_with_keys_ordered_in_alphabetical_order",
        serialize_with = "serialize_program_to_base64"
    )]
    pub program: Value,
    pub abi: Vec<ContractClassAbiEntryWithType>,
    /// The selector of each entry point is a unique identifier in the program.
    pub entry_points_by_type: LegacyEntryPointsByType,
}

impl DeprecatedContractClass {
    pub fn rpc_from_json_str(json_str: &str) -> DevnetResult<DeprecatedContractClass> {
        let res: DeprecatedContractClass =
            serde_json::from_str(json_str).map_err(JsonError::SerdeJsonError)?;

        Ok(res)
    }

    pub fn rpc_from_path(path: &str) -> DevnetResult<DeprecatedContractClass> {
        let contract_class_str = std::fs::read_to_string(path)?;
        DeprecatedContractClass::rpc_from_json_str(&contract_class_str)
    }
}

impl PartialEq for DeprecatedContractClass {
    fn eq(&self, other: &Self) -> bool {
        self.program == other.program && self.abi == other.abi
    }
}

impl Eq for DeprecatedContractClass {}

impl Default for DeprecatedContractClass {
    fn default() -> Self {
        Self {
            program: Value::default(),
            abi: Vec::new(),
            entry_points_by_type: LegacyEntryPointsByType {
                constructor: Vec::new(),
                external: Vec::new(),
                l1_handler: Vec::new(),
            },
        }
    }
}

impl HashProducer for DeprecatedContractClass {
    fn generate_hash(&self) -> DevnetResult<Felt> {
        let json_value: Cairo0Json = self.clone().try_into()?;
        json_value.generate_hash()
    }
}

impl TryFrom<DeprecatedContractClass> for StarknetInRustContractClass {
    type Error = Error;
    fn try_from(value: DeprecatedContractClass) -> Result<Self, Self::Error> {
        let json_value: Cairo0Json = value.try_into()?;
        let starknet_in_rust_contract_class =
            StarknetInRustContractClass::from_str(&json_value.inner.to_string())
                .map_err(|err| JsonError::Custom { msg: err.to_string() })?;

        Ok(starknet_in_rust_contract_class)
    }
}

impl TryInto<CompressedLegacyContractClass> for DeprecatedContractClass {
    type Error = Error;
    fn try_into(self) -> Result<CompressedLegacyContractClass, Self::Error> {
        // TODO: improve
        let cairo0: Cairo0Json = self.try_into()?;
        cairo0.try_into()
    }
}

#[cfg(test)]
mod tests {
    use crate::contract_class::DeprecatedContractClass;
    use crate::utils::test_utils::CAIRO_0_RPC_CONTRACT_PATH;
    use starknet_rs_core::types::CompressedLegacyContractClass;

    #[test]
    fn test_rpc_deserialization() {
        let _ = DeprecatedContractClass::rpc_from_path(CAIRO_0_RPC_CONTRACT_PATH).unwrap();
    }

    #[test]
    fn test_rpc_to_codegen() {
        let contract_class =
            DeprecatedContractClass::rpc_from_path(CAIRO_0_RPC_CONTRACT_PATH).unwrap();

        let _: CompressedLegacyContractClass = contract_class.try_into().unwrap();
    }
}
