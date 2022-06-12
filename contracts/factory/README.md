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
            * [GetAMMSettings](#GetAMMSettings)
    * [Hooks](#Hook)
        * Messages
            * [RegisterAMMPair](#RegisterAMMPair)

# Introduction
Contract responsible for initializing AMM Pairs. Any Router that points to this factory will consider the pairs on this contract to be verified.

# Sections
## Init
|Name|Type|Description|Optional|
|-|-|-|-|
|pair_contract|ContractInstantiationInfo|Stored contract information used to initialize new instances of the Pair Contract||
|amm_settings|AMMSettings<HumanAddr>|Settings used for the AMM Pairs regarding the lp_fee, the shade_dao_fee and the shade_dao_address. This is queried real-time on every trade directly on the factory address||
|lp_token_contract|ContractInstantiationInfo|Stored contract information used to initialize new instances of the Pair Contract|
|prng_seed|Binary|This seed is passed to all the pair contracts instantiated from the factory||

# Admin
## Messages
### SetConfig

Sets the configuration of the Factory Contract

|Name|Type|Description|Optional|
|-|-|-|-|
|pair_contract|If value is present, update the stored contract reference used to initialize new pair contracts||Yes|
|lp_token_contract|If value is present, update the stored contract reference used to initialize new lp tokens during pair contract intialization||Yes|
|amm_settings|If value is present, update the amm settings in the system||Yes|

### CreateAMMPair

Uses the factory to initialize a new AMM Pair Contract

|Name|Type|Description|Optional|
|-|-|-|-|
|pair|TokenPair<HumanAddr>|TokenPair used for the initialized pair contract|No|
|entropy|Binary|Entropy passed to the initialized pair contract|No|

### AddAMMPairs

Adds an existing AMM Pair Contract to the Factory

|Name|Type|Description|Optional|
|-|-|-|-|
|amm_pair|Vec<AMMPair<HumanAddr>>|Vector of AMM Pairs to register against the Factory|No|

# User
## Queries
### GetConfig

Gets the config of the factory contract

|Name|Type|Description|Optional|
|-|-|-|-|
|||||

### GetAMMPairAddress

Gets the AMM Pair Address

|Name|Type|Description|Optional|
|-|-|-|-|
|pair|TokenPair<HumanAddr>|Token Pair to look up in the registered pair in the factory|No|

### ListAMMPairs

Lists the AMM Pair Contracts registered with the Factory

|Name|Type|Description|Optional|
|-|-|-|-|
|pagination||||

### GetAMMSettings

Gets the current AMM Settings registered with the factory

|Name|Type|Description|Optional|
|-|-|-|-|
|||||

# Hook
## Messages
### RegisterAMMPair

Callback used by factory initialized contract addresses to register against the factory.

|Name|Type|Description|Optional|
|-|-|-|-|
|pair|TokenPair<HumanAddr>|Token Pair being registered|No|
|Signature|Binary|The signature used to verify the callback|No|