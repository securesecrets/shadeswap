use cosmwasm_std::Uint128;
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

use crate::token_type::TokenType;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenAmount<A> {
    pub token: TokenType<A>,
    pub amount: Uint128
}

impl<A: Clone> TokenAmount<A> {
    pub fn assert_sent_native_token_balance(&self, env: &Env) -> StdResult<()> {
        self.token.assert_sent_native_token_balance(env, self.amount)
    }
}