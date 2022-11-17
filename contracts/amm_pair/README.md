# AMM Pair Contract
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)
    * [Admin](#Admin)
        * Messages
            * [AddWhiteListAddress](#AddWhiteListAddress)
            * [RemoveWhitelistAddresses](#RemoveWhitelistAddresses)     
            * [SetCustomPairFee](#SetCustomPairFee) 
            * [SetConfig](#SetConfig)        
            * [RecoverFunds](#RecoverFunds)                
    * [User](#User)
        * Messages
            * [Receive](#Receive)  
            * [AddLiquidityToAMMContract](#AddLiquidityToAMMContract)
            * [SwapTokens](#SwapTokens)
            * [SetViewingKey](#SetViewingKey)
        * Queries
            * [GetPairInfo](#GetPairInfo)
            * [GetTradeHistory](#GetTradeHistory)  
            * [GetConfig](#GetConfig)  
            * [GetWhiteListAddress](#GetWhiteListAddress)  
            * [GetTradeCount](#GetTradeCount)             
            * [GetEstimatedPrice](#GetEstimatedPrice)
            * [GetEstimatedLiquidity](#GetEstimatedLiquidity)
    * [Invoke]
        * Messages
            * [SwapTokens](#SwapTokens(Callback))
            * [RemoveLiquidity](#RemoveLiquidity)


# Introduction
The Contract to hold Pair Between Swap Tokens.

# Sections

## Init
##### Request
| Name              | Type                             | Description                                                                | optional |
|-------------------|----------------------------------|----------------------------------------------------------------------------|----------|
| pair              | TokenPair                        | Token Pair to hold two token                                               | no       |
| lp_token_contract | ContractInstantiationInfo        | ContractInstantiationInfo                                                  | no       |
| factory_info      | Contract                     | Factory to manage this pair moving forwards                                          | yes       |
| prng_seed         | Binary                           | seed to use for viewing key                                                | no       |
| entropy           | Binary                           | Use to calculate viewing key                                               | no       |
| admin_auth             | Contract                        | Set the admin of AMMPair Contract                                          | no      |
| custom_fee             | CustomFee                        | The fee for the AMMPair, set to none to inherit fee from Factory         | yes      |
| staking_contract  | StakingContractInit              | Staking Contract Init Config                                               | yes      |

## Admin

### Messages

#### AddWhiteListAddress
Add address to whitelist, group of addresses which fee doesn't apply.

##### Request
| Name    | Type      | Description                                   | optional |
|---------|-----------|-----------------------------------------------|----------|
| address | String | The address to add to whitelist               | no       |

##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
```


#### RemoveWhitelistAddresses
Address to remove from whitelist.

##### Request
| Name    | Type      | Description                                   | optional |
|---------|-----------|-----------------------------------------------|----------|
| addresses | Vec<String> | The addresses to remove from whitelist          | no       |

##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
```

#### SetCustomPairFee
Set Custom Pair Fee to be used in Pair Contract.

##### Request
| Name    | Type      | Description                                   | optional |
|---------|-----------|-----------------------------------------------|----------|
| custom_fee | CustomFee | Custom Shade Dao and LP Fees          | yes       |

##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
```

#### SetConfig
Set the Admin contract.

##### Request
| Name    | Type      | Description                                   | optional |
|---------|-----------|-----------------------------------------------|----------|
| admin_auth | Contract | The admin authentication contract          | yes       |

##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
```
#### RecoverFunds
Recover Funds for Address.

##### Request
| Name    | Type      | Description                                   | optional |
|---------|-----------|-----------------------------------------------|----------|
| token | TokenType | Token type of token to be recovered         | no       |
| amount | Unit128 | The amount         | no       |
| to | String | The address to send the amount to         | no       |
| msg | Binary | Message to pass in the send         | yes       |
##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
```


## User

### Queries

#### GetPairInfo
Get information about the token pair.

##### Request
| Name    | Type   | Description                                   | optional |
|---------|--------|-----------------------------------------------|----------|
|         |        |                                               |          |

##### Response
```json
{
  "liquidity_token": "LP Token Contract",
  "factory": "Factory Contract",
  "pair": "Token Pair with two Token Type",
  "amount_0": "Balance of Token 0",
  "amount_1": "Balance of Token 1",
  "total_liquidity": "Total liquidity of pool",
  "contract_version": "Contract Version of the Smart Contract"
}
```


#### GetEstimatedPrice
Get Estimated Price for amount.

##### Request
| Name    | Type   | Description                                   | optional |
|---------|--------|-----------------------------------------------|----------|
|  offer  | TokenAmount   | amount for price estimation    |  no  |
|  exclude_fee  | bool   | exclude fee in price estimation    |  yes  |

##### Response
```json
{
  "estimated_price": "String",
}
```


#### GetTradeHistory
Get Information about trade history.

##### Request
| Name       | Type        | Description                              | optional |
|------------|-------------|------------------------------------------|----------|
| api_key | String  |     The API key to authenticate   |    no    |
| pagination | Pagination  | The start and limit   |    no    |

##### Response
```json
{
  "data": "[array of trade history]",
}
```
###### Where TradeHistory
```json
{
  "pair": "TokenPair",
  "lp_token_contract": "ContractInstantiationInfo",
  "factory_info": "Contract",
  "prng_seed": "Binary",
  "entropy": "Binary",
  "admin_auth": "Contract",
  "staking_contract": " Option<StakingContractInit>",
  "custom_fee": "Option<CustomFee>",
  "callback": "Option<Callback>",
}
```

#### GetTradeCount
Get Count of trade for pair contract.

##### Request
| Name       | Type        | Description                              | optional |
|------------|-------------|------------------------------------------|----------|
|            |             |                                          |          |

##### Response
```json
{
  "count": "trade count",
}
```

#### GetConfig
Get Configuration of AMMPair Contract.

##### Request
| Name       | Type        | Description                              | optional |
|------------|-------------|------------------------------------------|----------|
|            |             |                                          |          |

##### Response
```json
{
  "factory_contract": "Contract",
  "lp_token": "Contract",
  "staking_contract": "Option<Contract>",
  "pair": "TokenPair",
  "custom_fee": "Option<CustomFee>",
}
```


#### GetEstimatedLiquidity
Get Estimated Liquidity.

##### Request
| Name       | Type        | Description                              | optional |
|------------|-------------|------------------------------------------|----------|
|  deposit    |  TokenPairAmount    |     Token Pair to deposit                  |     no   |


##### Response
```json
{
  "lp_token": "Uint128",
  "total_lp_token": "Uint128",
}
```


#### GetWhiteListAddress
Get All addresses from whitelist.

##### Request
| Name       | Type        | Description                              | optional |
|------------|-------------|------------------------------------------|----------|
|      |    |                          |        |

##### Response
```json
{
  "amount": "Get all whitelist's addresses",
}
```

### Messages

#### SwapTokens
Swap Native Tokens.

##### Request

| Name      | Type        | Description                             | optional |
|-----------|----------------|--------------------------------------|----------|
| offer     | TokenAmount | Amount and Token Type                   | no       |
| expected_return | Uint128 | Slippage, amount willing to accept    | yes      |
| to | String | The address to remove from LP                  | yes       |

##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
```
#### SetViewingKey
Update the viewing Key for a Pair.

##### Request

| Name      | Type        | Description                             | optional |
|-----------|----------------|--------------------------------------|----------|
| viewing_key     | String | The viewing key                   | no       |


##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
```

#### Receive
Extension of the SNIP20 receive callback used when receiving SNIP20 tokens used for trades.


##### Request

|Name|Type|Description|Optional|
|-|-|-|-|
| from | String | who invokes the callback                  | no      |
| amount | Uint128 | amount sent               | no       |
| msg | Binary | Message to Invoke in Pair Contract                  | yes       |


##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
```

#### AddLiquidityToAMMContract
Add Liquidity to the Pool and Staking Contract if configured.

##### Request

| Name      | Type        | Description                             | optional |
|-----------|----------------|--------------------------------------|----------|
| deposit     | TokenPairAmount | Amount and Token Type             | no       |
| expected_return | Uint128 | slippage, amount willing to accept       | yes      |
| staking | bool | Add his LP token to Staking if it is allowed        | yes      |

##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
```


## Invoke
### Messages

#### SwapTokens(Callback)
Swap tokens.

##### Request

| Name      | Type        | Description                             | optional |
|-----------|----------------|--------------------------------------|----------|
| to | String | who invokes the callback                  | yes      |
| expected_return | Uint128 | Slippage, amount willing to accept                | yes       |



##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
```

#### RemoveLiquidity
Remove liquidity for address and remove from staking if applicable.

##### Request

| Name      | Type        | Description                             | optional |
|-----------|----------------|--------------------------------------|----------|
| from | String | address to remove liquidity             | yes      |

##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
```
