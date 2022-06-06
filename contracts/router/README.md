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
            * [SwapCallBack]
    * [Hooks]
        * Messages
            * [SwapTokensForExact]

# Introduction
The Router is stateless and can be replaced safely. This is to ensure upgradability of functionality with minimal impact.
Stateful data is stored primarily within the factory.

# Sections

## Init

## Best Path
Best path is calculated within the client, when invoking a swap that path is then provided to the router.

