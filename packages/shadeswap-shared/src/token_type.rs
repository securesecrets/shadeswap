use cosmwasm_std::{
    from_binary,
    Api,
    Binary,
    Extern,
    HumanAddr,
    Querier,
    StdError,
    StdResult,
    Storage, Env, HandleResponse, log, CanonicalAddr, Uint128, CosmosMsg, WasmMsg, to_binary, BankMsg, Coin,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use secret_toolkit::snip20::{balance_query};
use secret_toolkit::snip20::HandleMsg::Send;

use crate::core::{Canonize, Humanize};

const BLOCK_SIZE: usize = 256;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TokenType<A> {
    CustomToken {
        contract_addr: A,
        token_code_hash: String,
        //viewing_key: String,
    },
    NativeToken {
        denom: String,
    },
}

#[deprecated(note = "please use TokenType<CanonicalAddr> instead")]
pub type TokenTypeStored = TokenType<CanonicalAddr>;


impl Canonize for TokenType<HumanAddr> {
    fn canonize(self, api: &impl Api) -> StdResult<TokenType<CanonicalAddr>> {
        Ok(match self {
            Self::CustomToken {
                contract_addr,
                token_code_hash,
            } => TokenType::CustomToken {
                contract_addr: contract_addr.canonize(api)?,
                token_code_hash: token_code_hash.clone(),
            },
            Self::NativeToken { denom } => TokenType::NativeToken {
                denom: denom.clone(),
            },
        })
    }

    type Output = TokenType<CanonicalAddr>;
}
impl Humanize for TokenType<CanonicalAddr> {
    fn humanize(self, api: &impl Api) -> StdResult<TokenType<HumanAddr>> {
        Ok(match self {
            Self::CustomToken {
                contract_addr,
                token_code_hash,
            } => TokenType::CustomToken {
                contract_addr: contract_addr.humanize(api)?,
                token_code_hash: token_code_hash.clone(),
            },
            Self::NativeToken { denom } => TokenType::NativeToken {
                denom: denom.clone(),
            },
        })
    }

    type Output = TokenType<HumanAddr>;
}

impl<A: Clone> TokenType<A> {
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
    pub fn assert_sent_native_token_balance(&self, env: &Env, amount: Uint128) -> StdResult<()> {
        if let TokenType::NativeToken { denom } = &self {
            return match env.message.sent_funds.iter().find(|x| x.denom == *denom) {
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

impl TokenType<HumanAddr> {
    pub fn query_balance(
        &self,
        querier: &impl Querier,
        exchange_addr: HumanAddr,
        viewing_key: String,
    ) -> StdResult<Uint128> {
        match self {
            TokenType::NativeToken { denom } => {
                let result = querier.query_balance(exchange_addr, denom)?;
                Ok(result.amount)
            }
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
            } => {
                let result = balance_query(
                    querier,
                    exchange_addr,
                    viewing_key,
                    BLOCK_SIZE,
                    token_code_hash.clone(),
                    contract_addr.clone(),
                )?;
                Ok(result.amount)
            }
        }
    }

    pub fn create_send_msg (
        &self,
        sender: HumanAddr,
        recipient: HumanAddr,
        amount: Uint128
    ) -> StdResult<CosmosMsg> {
        let msg = match self {
            TokenType::CustomToken { contract_addr, token_code_hash } => {
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: contract_addr.clone(),
                    callback_code_hash: token_code_hash.to_string(),
                    msg: to_binary(&Send {  
                        recipient,
                        amount,
                        padding: None,
                        msg: None,
                        recipient_code_hash: None,
                        memo: None,
                    })?,
                    send: vec![]
                })
            },
            TokenType::NativeToken { denom } => {            
                CosmosMsg::Bank(BankMsg::Send {
                    from_address: sender,
                    to_address: recipient,
                    amount: vec![Coin {
                        denom: denom.clone(),
                        amount
                    }],
                })
            }
        };
    
        Ok(msg)
    }
}
