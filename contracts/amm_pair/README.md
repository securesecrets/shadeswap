# AMM Pair Contract
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)
    * [Admin](#Admin)
        * Messages
            * [AddWhiteListAddress](#AddWhiteListAddress)
            * [RemoveWhitelistAddresses](#RemoveWhitelistAddresses)    
            * [RemoveLiquidity](#RemoveLiquidity)                     
    * [User](#User)
        * Messages
            * [GetPairInfo](#GetPairInfo)
            * [SwapTokensForExact]
            * [SwapCallBack]
        * Queries
            * [GetPairInfo](#GetPairInfo)
            * [GetTradeHistory](#GetTradeHistory)   
            * [GetAdmin](#GetAdmin)  
            * [GetWhiteListAddress](#GetWhiteListAddress)  
            * [GetTradeCount](#GetTradeCount)  
            * [GetClaimReward](#GetClaimReward)                          
    * [Hooks]
        * Messages
            * [SwapTokens]
            * [OnLpTokenInitAddr]

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


##Admin

### Messages

#### AddWhiteListAddress
Add address to whitelist, group of addresses which fee doesn't apply.

##### Request
| Name    | Type   | Description                                   | optional |
|---------|--------|-----------------------------------------------|----------|
| address | String | The address to add to whitelist               | no       |

##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}

#### RemoveWhitelistAddresses
Address to remove from whitelist.

##### Request
| Name    | Type   | Description                                   | optional |
|---------|--------|-----------------------------------------------|----------|
| address | String | The address to remove from whitelist          | no       |

##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}

#### RemoveLiquidity
Remove liquidity from Liquidity Pool (LP).

##### Request
| Name      | Type   | Description                                   | optional |
|-----------|--------|-----------------------------------------------|----------|
| recipient | String | The address to remove from LP                 | no       |

##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}

##User

### Messages

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


