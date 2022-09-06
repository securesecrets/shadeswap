use cosmwasm_std::{Uint128, MessageInfo};
use cosmwasm_std::{
    from_binary,
    Api,
    Binary,
    Querier,
    StdError,
    StdResult,
    Storage, Env, Response
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::TokenType;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenAmount {
    pub token: TokenType,
    pub amount: Uint128
}

impl TokenAmount {
    pub fn assert_sent_native_token_balance(&self, info: &MessageInfo) -> StdResult<()> {
        self.token.assert_sent_native_token_balance(info, self.amount)
    }
}