# Staking Contract
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)                  
    * [User](#User)
        * Messages       
            * [ClaimRewards](#ClaimRewards)
            * [SetVKForStaker](#SetVKForStaker)            
            * [Unstake](#Unstake)    
        * Queries
            * [GetContractOwner](#GetContractOwner)
            * [GetClaimReward](#GetClaimReward)   
            * [GetStakerLpTokenInfo](#GetStakerLpTokenInfo)
            * [GetRewardTokenBalance](#GetStakerLpTokenInfo)
            * [GetStakerRewardTokenBalance](#GetStakerLpTokenInfo)   
    * [Hooks](#Hooks)
        * Messages
            * [SetLPToken](#SetLPToken) 
            * [Receive](#Receive)
    * [Invoke]
        * Messages
            * [Stake](#Stake)            
    * [Callback]
        * Messages
            * [Callback](#Callback)

# Introduction
The Contract to hold Pair Between Swap Tokens.

# Sections

## Init
##### Request
| Name              | Type                             | Description                                                                | optional |
|-------------------|----------------------------------|----------------------------------------------------------------------------|----------|
| staking_amount    | Uint128     | Total Reward Amount for staking | no       |
| reward_token | TokenType   |   Reward Token Type              | no       |
| contract | ContractLink | AMMPair Contract Address Link to register staking contract  | no    |






## User

### Queries

#### GetContractOwner
Get Contract Owner Address.

##### Request
| Name       | Type        | Description                              | optional |
|------------|-------------|------------------------------------------|----------|


##### Response
```json
{
  "address": "Contract Owner Address",
}
```

#### GetClaimReward
Get Claimable Reward for staker.

##### Request
| Name       | Type        | Description                              | optional |
|------------|-------------|------------------------------------------|----------|
|   staker  | HumanAddr |  Address to calculate claimable amount      |   no |
|   time  | u128 |  Time to use for calculation claimable amount      |   no |
|   key  | String |  Key which user setup for viewing key      |   no |

##### Response
```json
{
  "amount": "trade count",
}
```

#### GetStakerLpTokenInfo
Get  Staker Lp Token Information.

##### Request
| Name       | Type        | Description                              | optional |
|------------|-------------|------------------------------------------|----------|
|   staker  | HumanAddr |  Address to calculate claimable amount      |   no |
|   key  | String |  Key which user setup for viewing key      |   no |

##### Response
```json
{
  "staked_lp_token": "Uint128",
  "total_staked_lp_token": "Uint128",
  "reward_token" : "ContractLink"
}
```

#### GetRewardTokenBalance
Get Reward Token Balance for staker

##### Request
| Name       | Type        | Description                              | optional |
|------------|-------------|------------------------------------------|----------|
|   staker  | HumanAddr |  Address to calculate claimable amount      |   no |
|   key  | String |  Seed which user setup for viewing key      |   no |

##### Response
```json
{
  "amount": "Uint128",
  "reward_token" : "ContractLink"
}
```

#### GetStakerRewardTokenBalance
Get Reward Token Balance for staker and total Reward Liquidity

##### Request
| Name       | Type        | Description                              | optional |
|------------|-------------|------------------------------------------|----------|
|   staker  | HumanAddr |  Address to calculate claimable amount      |   no |
|   key  | String |  Seed which user setup for viewing key      |   no |

##### Response
```json
{
  "reward_amount": "Uint128",
  "total_rewards_liquidity": "Uint128"
}
```

### Messages

#### ClaimRewards
Claim reward.

##### Request

| Name      | Type        | Description                             | optional |
|-----------|----------------|--------------------------------------|----------|

##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
```

#### Unstake
Remove amount and address from staking

##### Request
| Name    | Type      | Description                                   | optional |
|---------|-----------|-----------------------------------------------|----------|
| amount | Uint128 | Amount to unstake          | no       |
| remove_liquidity | bool | Remove form liquidity          | yes       |

##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
```

#### SetVKForStaker
Set viewing key for staker

##### Request
| Name    | Type      | Description                                   | optional |
|---------|-----------|-----------------------------------------------|----------|
| key | String |  Seed for viewing key          | no       |


##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
```


## Callback
### Messages

#### Receive
Receive Callback.

##### Request

| Name      | Type        | Description                             | optional |
|-----------|----------------|--------------------------------------|----------|
| from | HumanAddr | who invokes the callback                  | no      |
| msg |  | Message to Invoke in Pair Contract                  | yes       |
| amount | Uint128 | amount sent               | no       |

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

#### Stake
Stake amount and address

##### Request

| Name      | Type        | Description                             | optional |
|-----------|----------------|--------------------------------------|----------|
| from | HumanAddr | who invokes the callback |  no      |
| amount | Uint128 | amount sent |   no       |
   |
##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
```