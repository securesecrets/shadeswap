use cosmwasm_std::{Deps, StdResult, to_binary, Binary};
use shadeswap_shared::{Pagination, amm_pair::{AMMPair, generate_pair_key}, core::TokenPair, factory::QueryResponse};

use crate::state::{total_amm_pairs_r, PAGINATION_LIMIT, amm_pairs_r, amm_pair_keys_r};

pub fn amm_pairs_page(deps: Deps, pagination: Pagination) -> StdResult<Vec<AMMPair>> {
    let count = total_amm_pairs_r(deps.storage).may_load()?;

    match count {
        Some(c) => {
            if pagination.start >= c {
                return Ok(vec![]);
            }

            let limit = pagination.limit.min(PAGINATION_LIMIT);
            let end = (pagination.start + limit as u64).min(c);

            let mut result = Vec::with_capacity((end - pagination.start) as usize);

            for i in pagination.start..end {
                let exchange: AMMPair = amm_pairs_r(deps.storage).load(i.to_string().as_bytes())?;

                result.push(exchange);
            }

            Ok(result)
        }
        None => Ok(vec![]),
    }
}

pub fn amm_pair_address(deps: &Deps, pair: TokenPair) -> StdResult<Binary> {
    let address = amm_pair_keys_r(deps.storage).load(&generate_pair_key(&pair))?;
    to_binary(&QueryResponse::GetAMMPairAddress {
        address: address.to_string(),
    })
}

pub fn pairs_page(deps: Deps, pagination: Pagination) -> StdResult<Binary> {
    let amm_pairs = amm_pairs_page(deps, pagination)?;

    to_binary(&QueryResponse::ListAMMPairs { amm_pairs })
}


