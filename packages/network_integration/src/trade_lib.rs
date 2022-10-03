
pub mod trade_lib{
    use cosmwasm_std::Addr;
    use secretcli::cli_types::NetContract;
    use shadeswap_shared::core::TokenPair;

    use super::*;


    pub fn create_token_pair() -> TokenPair{
        let token_pair: TokenPair = TokenPair(
            shadeswap_shared::core::TokenType::CustomToken { 
                contract_addr: Addr::unchecked("secret1g3tm5taj293qkf33wpksvchk83jssknju92phl"), 
                token_code_hash: "d08d4acd6c1138d89180cfb4065208b041472abe7a3233b65005040f86fd500e".to_string() 
            },
            shadeswap_shared::core::TokenType::CustomToken { 
                contract_addr: Addr::unchecked("secret1nulgwu6es24us9urgyvms7y02txyg0s02msgzw"), 
                token_code_hash: "d08d4acd6c1138d89180cfb4065208b041472abe7a3233b65005040f86fd500e".to_string() 
            }
        );
        return token_pair
    }

    pub fn create_router_contract() -> NetContract{
        let router_contract: NetContract = NetContract{
            label: "".to_string(),
            id: "13135".to_string(),
            address: "secret1pf42ypa2awg0pxkx8lfyyrjvm28vq0qpffa8qx".to_string(),
            code_hash: "de4dd561389d6574289da86f8b24f46deba2f00190bb956451d3c24732b424ca".to_string(),
        };

        return router_contract
    }
}