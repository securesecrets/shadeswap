# FACTORY

## Getting Started

# Factory Contract
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)
    * [Admin](#Admin)
        * Messages
            * [SetConfig](#SetConfig)
            * [CreateAMMPair](#CreateAMMPair)
            * [AddAMMPairs](#AddAMMPairs)
    * [User](#User)
        * Queries
            * [GetConfig](#GetConfig)
            * [GetAMMPairAddress](#GetAMMPairAddress)
            * [ListAMMPairs](#ListAMMPairs)
            * [AuthorizeApiKey](#AuthorizeApiKey)
    * [Hooks](#Hook)
        * Messages
            * [RegisterAMMPair](#RegisterAMMPair)

# Introduction
Contract responsible for initializing AMM Pairs. Any Router that points to this factory will consider the pairs on this contract to be verified.

# Sections
## Init
|Name|Type|Description|Optional|
|-|-|-|-|
|pair_contract|ContractInstantiationInfo|Stored contract information used to initialize new instances of the Pair Contract|no|
|amm_settings|AMMSettings|Settings used for the AMM Pairs regarding the lp_fee, the shade_dao_fee and the shade_dao_address. This is queried real-time on every trade directly on the factory address|no|
|lp_token_contract|ContractInstantiationInfo|Stored contract information used to initialize new instances of the Pair Contract|
|prng_seed|Binary|This seed is passed to all the pair contracts instantiated from the factory|no|
|api_key|String|Stores the API key that will be used for authentication|no|
|authenticator|Contract|Set the default authenticator for all permits on the contracts|no|
|admin_auth|Contract|Set the admin|no|


# Admin
## Messages
### SetConfig

Sets the configuration of the Factory Contract

|Name|Type|Description|Optional|
|-|-|-|-|
|pair_contract|ContractInstantiationInfo|If value is present, update the stored contract reference used to initialize new pair contracts|yes|
|lp_token_contract|ContractInstantiationInfo|If value is present, update the stored contract reference used to initialize new lp tokens during pair contract intialization|yes|
|amm_settings|AMMSettings|If value is present, update the amm settings in the system|yes|
|api_key|String|Updates the API key that will be used for authentication|yes|
|admin_auth|Contract|Set the admin|yes|
#### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
```
### CreateAMMPair

Uses the factory to initialize a new AMM Pair Contract

|Name|Type|Description|Optional|
|-|-|-|-|
|pair|TokenPair<HumanAddr>|TokenPair used for the initialized pair contract|no|
|entropy|Binary|Entropy passed to the initialized pair contract|no|
|staking_contract|StakingContractInit|The staking contract and its configuration|yes|
|router_contract|Contract|This is used to optionally register the token|yes|

#### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
```


### AddAMMPairs

Adds an existing AMM Pair Contract to the Factory

|Name|Type|Description|Optional|
|-|-|-|-|
|amm_pair|Vec<AMMPair>|Vector of AMM Pairs to register against the Factory|no|

#### Response
```json
{
  "complete_task": {
    "status": "success"
  }
}
```


# User
## Queries
### GetConfig

Gets the config of the factory contract

|Name|Type|Description|Optional|
|-|-|-|-|
|||||

#### Response
```json
{
  "pair_contract": "ContractInstantiationInfo",
  "amm_settings": "AMMSettings",
  "            lp_token_contract: lp_token_contract,
": "ContractInstantiationInfo",
  "authenticator": "Option<Contract>",
  "admin_auth": "Contract",
}
```
### GetAMMPairAddress

Gets the AMM Pair Address

|Name|Type|Description|Optional|
|-|-|-|-|
|pair|TokenPair|Token Pair to look up in the registered pair in the factory|No|
#### Response
```json
{
  "address": "String",
}
```
### ListAMMPairs

Lists the AMM Pair Contracts registered with the Factory

|Name|Type|Description|Optional|
|-|-|-|-|
| pagination | Pagination  | The start and limit   |    no    |
#### Response
```json
{
  "amm_pairs": "[array of AMMPair]",
}
```
### AuthorizeApiKey

Gets the current AMM Settings registered with the factory

|Name|Type|Descriptiofn|Optional|
|-|-|-|-|
| api_key | String  |     The API key to check   |    no    |
#### Response
```json
{
  "authorized": "bool",
}
```

# Hook
## Messages
### RegisterAMMPair

Callback used by factory initialized contract addresses to register against the factory.

|Name|Type|Description|Optional|
|-|-|-|-|
|pair|TokenPair|Token Pair being registered|no|
|Signature|Binary|The signature used to verify the callback|no|

