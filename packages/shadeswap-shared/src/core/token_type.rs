use crate::snip20::ExecuteMsg::Send;
use cosmwasm_std::{
    from_binary, to_binary, Api, BankMsg, Binary, CanonicalAddr, Coin, CosmosMsg, Env, Querier,
    Response, StdError, StdResult, Storage, Uint128, WasmMsg, MessageInfo, DepsMut, Addr, Deps,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::contract_interfaces::snip20::helpers::{balance_query};
use crate::utils::asset::Contract;

const BLOCK_SIZE: usize = 256;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    CustomToken {
        contract_addr: Addr,
        token_code_hash: String,
        //viewing_key: String,
    },
    NativeToken {
        denom: String,
    },
}

impl TokenType {
    pub fn is_native_token(&self) -> bool {
        match self {
            TokenType::NativeToken { .. } => true,
            TokenType::CustomToken { .. } => false,
        }
    }
    pub fn is_custom_token(&self) -> bool {
        match self {
            TokenType::NativeToken { .. } => false,
            TokenType::CustomToken { .. } => true,
        }
    }
    pub fn assert_sent_native_token_balance(&self, info: &MessageInfo, amount: Uint128) -> StdResult<()> {
        if let TokenType::NativeToken { denom } = &self {
            return match info.funds.iter().find(|x| x.denom == *denom) {
                Some(coin) => {
                    if amount == coin.amount {
                        Ok(())
                    } else {
                        Err(StdError::generic_err("Native token balance mismatch between the argument and the transferred"))
                    }
                }
                None => {
                    if amount.is_zero() {
                        Ok(())
                    } else {
                        Err(StdError::generic_err("Native token balance mismatch between the argument and the transferred"))
                    }
                }
            };
        }

        Ok(())
    }
}

impl TokenType {
    pub fn query_balance(
        &self,
        deps: Deps,
        exchange_addr: String,
        viewing_key: String,
    ) -> StdResult<Uint128> {
        match self {
            TokenType::NativeToken { denom } => {
                let result = deps.querier.query_balance(exchange_addr, denom)?;
                Ok(result.amount)
            }
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
            } => {
                balance_query(&deps.querier,  deps.api.addr_validate(&exchange_addr)?, viewing_key,  &Contract {
                    address: contract_addr.clone(),
                    code_hash: token_code_hash.clone(),
                })
            }
        }
    }

    pub fn create_send_msg(
        &self,
        sender: String,
        recipient: String,
        amount: Uint128,
    ) -> StdResult<CosmosMsg> {
        let msg = match self {
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
            } => CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.clone().into_string(),
                code_hash: token_code_hash.to_string(),
                msg: to_binary(&Send {
                    recipient,
                    amount,
                    padding: None,
                    msg: None,
                    recipient_code_hash: None,
                    memo: None,
                })?,
                funds: vec![],
            }),
            TokenType::NativeToken { denom } => CosmosMsg::Bank(BankMsg::Send {
                to_address: recipient,
                amount: vec![Coin {
                    denom: denom.clone(),
                    amount,
                }],
            }),
        };
        Ok(msg)
    }
}
