

pub fn init(
    chain: &mut App,
    sender: &str,
    contracts: &mut DeployedContracts,
    name: &str,
    snip20_symbol: &str,
    decimals: u8,
    config: Option<snip20::InitConfig>,
) -> StdResult<()> {
    let snip20 = Contract::from(
        match (snip20::InstantiateMsg {
            name: name.to_string(),
            admin: Some(sender.into()),
            symbol: snip20_symbol.to_string(),
            decimals,
            initial_balances: Some(vec![snip20::InitialBalance {
                address: sender.into(),
                amount: Uint128::from(1_000_000_000 * 10 ^ decimals as u128),
            }]),
            prng_seed: Binary::default(),
            query_auth: None,
            config,
        }
        .test_init(
            Snip20::default(),
            chain,
            Addr::unchecked(sender),
            "snip20",
            &[],
        )) {
            Ok(contract_info) => contract_info,
            Err(e) => return Err(StdError::generic_err(e.to_string())),
        },
    );
    contracts.insert(
        SupportedContracts::Snip20(snip20_symbol.to_string()),
        snip20,
    );
    Ok(())
}