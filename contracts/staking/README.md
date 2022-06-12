# Staking Contract
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)
    * [Admin](#Admin)
        * Messages
            * [Stake](#Stake)
            * [Unstake](#Unstake)                       
    * [User](#User)
        * Messages       
            * [ClaimRewards](#ClaimRewards)
        * Queries
            * [GetStakers](#GetStakers)
            * [GetClaimReward](#GetClaimReward)   
            * [GetContractOwner](#GetAGetContractOwnerdmin)    

# Introduction
The Contract to hold Pair Between Swap Tokens.

# Sections

## Init
##### Request
| Name              | Type                             | Description                                                                | optional |
|-------------------|----------------------------------|----------------------------------------------------------------------------|----------|
| staking_amount    | Uint128     | Total Reward Amount for staking | no       |
| reward_token | TokenType   |   Reward Token Type              | no       |
| code_hash | String | AMMPair code hash for register staking contract  | no    |


## Admin

### Messages

#### Stake
Add address to staking

##### Request
| Name    | Type      | Description                                   | optional |
|---------|-----------|-----------------------------------------------|----------|
| from  | HumanAddr | The address to add to staking               | no       |
| amount  | Uint128 | staking amount               | no       |

##### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
```


#### Unstake
Remove address from staking

##### Request
| Name    | Type      | Description                                   | optional |
|---------|-----------|-----------------------------------------------|----------|
| address | HumanAddr | The address to remove from staking          | no       |

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

#### GetStakers
Get list of all stakers.

##### Request
| Name    | Type   | Description                                   | optional |
|---------|--------|-----------------------------------------------|----------|

##### Response
```json
{
  "stakers": "[array of HumanAddr]",
}
```

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

##### Response
```json
{
  "count": "trade count",
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
