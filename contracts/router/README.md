# Router Contract
* [Introduction](#Introduction)
* [Sections](#Sections)
    * [Init](#Init)
    * [Admin](#Admin)
        * Messages
            * [RegisterSNIP20Token]
        * Queries
    * [User](#User)
        * Messages
            * [Receive]
            * [SwapTokensForExact]
    * [Hooks](#Hooks)
        * Messages
            * [SwapCallBack]
    * [Invoke](#Invoke)
        * Messages
            * [SwapTokensForExact]

# Introduction
The Router is stateless between transactions and can be replaced safely except for view keys specific to the SNIP20 to be traded. Before swapping the router contract, make sure all SNIP20s to be traded in the new contract are registered. This is to ensure upgradability of functionality with minimal impact.
Stateful data is stored within the factory.

# Sections
## Init

Initialize a Router Contract

|Name|Type|Description|Optional|
|-|-|-|-|
|factory_address|ContractLink|The Factory Contract to register the router for|No|
|prng_seed|Binary|Seed used for generated viewing key|No|
|entropy|Binary|Entropy used for generated viewing key|No|
|viewing_key|Option<ViewingKey>|Fixed viewing key to use for router, useful for testing|No|

## Admin
### Messages
#### RegisterSNIP20Token

Register the router's viewing key with SNIP20 contract. This is required to verify the amount of tokens that the router contract receives on each step of a swap.

|Name|Type|Description|Optional|
|-|-|-|-|
|token|HumanAddr|Register the viewing key for the router to the SNIP20 Token Contract|No|
|token_code_hash|String|Token code hash used to verify the contract that is being registered|No|

## User
### Messages
#### Receive

Extension of the SNIP20 receive callback used when receiving SNIP20 tokens used for trades.

|Name|Type|Description|Optional|
|-|-|-|-|
|from|HumanAddr||No|
|msg|Option<Binary>||No|
|amount|String||No|

#### SwapTokensForExact

Used to trade the native token. Calls to this interface directly sending a SNIP20 token will not work, instead use the SNIP20 send with a embedded invoke.

|Name|Type|Description|Optional|
|-|-|-|-|
|offer|HumanAddr|The native token amount sent into the start of the router trade|No|
|expected_return|Option<Binary>|When given, the minimum amount of tokens that need to come out of the router trade|Yes|
|path|Vec<HumanAddr>|The pair addresses in a array used for each leg of the trade|No|
|recipient|Option<HumanAddr>|Specify a recepient besides the sender of the native token|Yes|
## Hooks
### Messages
#### SwapCallBack

Swap callback is called by the pair contract after completing a trade to initialize the next step in the trade.

|Name|Type|Description|Optional|
|-|-|-|-|
|last_token_out|TokenAmount<HumanAddr>|The token coming out from the pair contract trade|No|
|signature|Binary|Signature to verify correct contract is calling back|No|

## Invoke
### Messages
#### SwapTokensForExact

Used with SNIP20 Send message to initiate router swap.

|Name|Type|Description|Optional|
|-|-|-|-|
|expected_return|Option<Binary>|When given, the minimum amount of tokens that need to come out of the router trade|Yes|
|path|Vec<HumanAddr>|The pair addresses in a array used for each leg of the trade|No|
|recipient|Option<HumanAddr>|Specify a recepient besides the sender of the SNIP20 token|No|

## Best Path
Best path is calculated within the client, when invoking a swap that path is then provided to the router.

