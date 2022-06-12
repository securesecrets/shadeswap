# AMM Pair Contract
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)
    * [Admin](#Admin)
        * Messages
            * [AddWhiteListAddress](#AddWhiteListAddress)
            * [RemoveWhitelistAddresses](#RemoveWhitelistAddresses)   
            * [SetAMMPairAdmin](#SetAMMPairAdmin)                     
    * [User](#User)
        * Messages       
            * [SwapTokens](#SwapTokens)
            * [AddLiquidityToAMMContract](#AddLiquidityToAMMContract)
        * Queries
            * [GetPairInfo](#GetPairInfo)
            * [GetTradeHistory](#GetTradeHistory)   
            * [GetAdmin](#GetAdmin)  
            * [GetWhiteListAddress](#GetWhiteListAddress)  
            * [GetTradeCount](#GetTradeCount)  
            * [GetClaimReward](#GetClaimReward)                          
    * [Hooks]
        * Messages
            * [Receive](#Receive)
            * [OnLpTokenInitAddr](#OnLpTokenInitAddr)
    * [Invoke]
        * Messages
            * [SwapTokens](#SwapTokens)
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
| factory_info      | ContractLink                     | The token that will be airdropped                                          | no       |
| prng_seed         | Binary                           | seed to use for viewing key                                                | no       |
| callback          | Callback                         | Callback to AmmPair Contract to register LP Token                          | yes      |
| entropy           | Binary                           | Use to calculate viewing key                                               | no       |
| admin             | HumanAddr                        | Set the admin of AMMPair Contract                                          | yes      |
| staking_contract  | StakingContractInit              | Staking Contract Init Config                                               | yes      |


## Admin

### Messages

#### AddWhiteListAddress
Add address to whitelist, group of addresses which fee doesn't apply.

##### Request
| Name    | Type      | Description                                   | optional |
|---------|-----------|-----------------------------------------------|----------|
| address | HumanAddr | The address to add to whitelist               | no       |

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
| address | HumanAddr | The address to remove from whitelist          | no       |

##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
```

#### RemoveLiquidity
Remove liquidity from Liquidity Pool (LP).

##### Request

| Name      | Type      | Description                                   | optional |
|-----------|-----------|-----------------------------------------------|----------|
| recipient | HumanAddr | The address to remove from LP                 | no       |

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
  "liquidity_token": "LP Token ContractLink",
  "factory": "Factory ContractLink",
  "pair": "Token Pair with two Token Type",
  "amount_0": "Balance of Token 0",
  "amount_1": "Balance of Token 1",
  "total_liquidity": "Total liquidity of pool",
  "contract_version": "Contract Version of the Smart Contract"
}
```

#### GetTradeHistory
Get Information about trade history.

##### Request
| Name       | Type        | Description                              | optional |
|------------|-------------|------------------------------------------|----------|
| pagination | Pagination  |                                          |    no    |

##### Response
```json
{
  "data": "[array of trade history]",
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

#### GetAdmin
Get Admin Address of AMMPair Contract.

##### Request
| Name       | Type        | Description                              | optional |
|------------|-------------|------------------------------------------|----------|
|            |             |                                          |          |

##### Response
```json
{
  "address": "Admin Address",
}
```


#### GetClaimReward
Get Claimable Reward Amount for staking.

##### Request
| Name       | Type        | Description                              | optional |
|------------|-------------|------------------------------------------|----------|
|  staker    | HumanAddr   | staker's address                         |    no    |

##### Response
```json
{
  "amount": "Claimable Reward Amount",
}
```


#### GetWhiteListAddress
Get All addresses from whitelist.

##### Request
| Name       | Type        | Description                              | optional |
|------------|-------------|------------------------------------------|----------|
|  staker    | HumanAddr   | staker's address                         |    no    |

##### Response
```json
{
  "amount": "Get all whitelist's addresses",
}
```

### Messages

#### SwapTokens
Swap Native Token.

##### Request

| Name      | Type        | Description                             | optional |
|-----------|----------------|--------------------------------------|----------|
| offer     | TokenAmount | Amount and Token Type                   | no       |
| expected_return | Uint128 | slippage, amount willing to accept    | yes      |
| to | HumanAddr | The address to remove from LP                  | yes       |
| router_link | ContractLink | Router Contract Info               | yes       |
| callback_signature | Binary | signature to verify snip20        | yes       |
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
| slippage | Uint128 | slippage, amount willing to accept         | yes      |

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

#### SwapTokens
Swap Native Token.

##### Request

| Name      | Type        | Description                             | optional |
|-----------|----------------|--------------------------------------|----------|
| expected_return | Uint128 | slippage, amount willing to accept    | yes      |
| to | HumanAddr | The address to remove from LP                  | yes       |
| router_link | ContractLink | Router Contract Info               | yes       |
| callback_signature | Binary | signature to verify snip20        | yes       |
##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
```

#### RemoveLiquidity
Remove liqudity for address and remove from staking if applicable.

##### Request

| Name      | Type        | Description                             | optional |
|-----------|----------------|--------------------------------------|----------|
| recipient | HumanAddr | address to remove liqudity             | no      |
##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
```