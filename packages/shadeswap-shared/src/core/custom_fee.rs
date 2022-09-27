use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Clone, Copy, Debug)]
pub struct Fee {
    pub nom: u8,
    pub denom: u16,
}

impl Fee {
    pub fn new(nom: u8, denom: u16) -> Self {
        Self { nom, denom }
    }
}


#[derive(Serialize, Deserialize, Clone,  Debug, PartialEq, JsonSchema)]
pub struct CustomFee {
    pub shade_dao_fee: Fee,
    pub lp_fee: Fee
}