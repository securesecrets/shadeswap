

use cosmwasm_std::{Env, CanonicalAddr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::impl_canonize_default;

use super::{Canonize, Humanize};

pub type CodeId   = u64;
pub type CodeHash = String;

/// Info needed to instantiate a contract.
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ContractInstantiationInfo {
    pub code_hash: CodeHash,
    pub id:        CodeId
}

impl_canonize_default!(ContractInstantiationInfo);

// Disregard code hash because it is case insensitive.
// Converting to the same case first and the comparing is unnecessary
// as providing the wrong code hash when calling a contract will result
// in an error regardless and we have no way of checking that here.
impl PartialEq for ContractInstantiationInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// Info needed to talk to a contract instance.
#[derive(Default, Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ContractLink<A> {
    pub address:   A,
    pub code_hash: CodeHash
}

impl Canonize for ContractLink<String>
{
    type Output = ContractLink<CanonicalAddr>;

    fn canonize(self, api: &impl cosmwasm_std::Api) -> cosmwasm_std::StdResult<Self::Output> {
        Ok(ContractLink{ address: api.addr_canonicalize(&self.address)?, code_hash: self.code_hash })
    }
}

impl Humanize for ContractLink<CanonicalAddr>
{
    type Output = ContractLink<String>;

    fn humanize(self, api: &impl cosmwasm_std::Api) -> cosmwasm_std::StdResult<Self::Output> {
        Ok(ContractLink{ address: api.addr_humanize(&self.address)?.as_str().to_string(), code_hash: self.code_hash })
    }
}

// Disregard code hash because it is case insensitive.
// Converting to the same case first and the comparing is unnecessary
// as providing the wrong code hash when calling a contract will result
// in an error regardless and we have no way of checking that here.
impl<A: PartialEq> PartialEq for ContractLink<A> {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address
    }
}

impl From<Env> for ContractLink<String> {
    fn from (env: Env) -> ContractLink<String> {
        ContractLink {
            address:   env.contract.address.to_string(),
            code_hash: env.contract.code_hash,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq() {
        assert_eq!(
            ContractInstantiationInfo {
                id: 1,
                code_hash: "c1dc8261059fee1de9f1873cd1359ccd7a6bc5623772661fa3d55332eb652084".into()
            }, 
            ContractInstantiationInfo {
                id: 1,
                code_hash: "c1dc8261059fee1de9f1873cd1359ccd7a6bc5623772661fa3d55332eb652084".into()
            }
        );

        assert_eq!(
            ContractInstantiationInfo {
                id: 1,
                code_hash: "c1dc8261059fee1de9f1873cd1359ccd7a6bc5623772661fa3d55332eb652084".into()
            }, 
            ContractInstantiationInfo {
                id: 1,
                code_hash: "C1dc8261059fee1de9f1873cd1359ccd7a6bc5623772661fa3d55332eb652084".into()
            }
        );

        assert_ne!(
            ContractInstantiationInfo {
                id: 1,
                code_hash: "c1dc8261059fee1de9f1873cd1359ccd7a6bc5623772661fa3d55332eb652084".into()
            }, 
            ContractInstantiationInfo {
                id: 2,
                code_hash: "C1dc8261059fee1de9f1873cd1359ccd7a6bc5623772661fa3d55332eb652084".into()
            }
        );

        assert_eq!(
            ContractLink {
                address: String::from("secret1rgm2m5t530tdzyd99775n6vzumxa5luxcllml4"),
                code_hash: "c1dc8261059fee1de9f1873cd1359ccd7a6bc5623772661fa3d55332eb652084".into()
            },
            ContractLink {
                address: String::from("secret1rgm2m5t530tdzyd99775n6vzumxa5luxcllml4"),
                code_hash: "c1dc8261059fee1de9f1873cd1359ccd7a6bc5623772661fa3d55332eb652084".into()
            }
        );

        assert_eq!(
            ContractLink {
                address: String::from("secret1rgm2m5t530tdzyd99775n6vzumxa5luxcllml4"),
                code_hash: "c1dc8261059fee1de9f1873cd1359ccd7a6bc5623772661fa3d55332eb652084".into()
            },
            ContractLink {
                address: String::from("secret1rgm2m5t530tdzyd99775n6vzumxa5luxcllml4"),
                code_hash: "C1dc8261059fee1de9f1873cd1359ccd7a6bc5623772661fa3d55332eb652084".into()
            }
        );

        assert_ne!(
            ContractLink {
                address: String::from("secret1rgm2m5t530tdzyd99775n6vzumxa5luxcllml4"),
                code_hash: "c1dc8261059fee1de9f1873cd1359ccd7a6bc5623772661fa3d55332eb652084".into()
            },
            ContractLink {
                address: String::from("secret1rgm2m5t530tdzyd99775n6vzumxa5luxcllml5"),
                code_hash: "C1dc8261059fee1de9f1873cd1359ccd7a6bc5623772661fa3d55332eb652084".into()
            }
        );
    }
}
