# FACTORY

## Getting Started
To compile

`make`

`docker run -it --rm -p 26657:26657 -p 26656:26656 -p 1337:1337 -v %cd%:/root/code --name secretdev enigmampc/secret-network-sw-dev`

In another windows

`docker exec -it secretdev /bin/bash`

Instinatiate the code
`secretd tx compute store contract.wasm --from a --gas 1000000 -y --keyring-backend test`

View code
`secretd query compute list-code`


`INIT='{"count": 100000000}'`
`CODE_ID=1`
`secretcli tx compute instantiate $CODE_ID "$INIT" --from a --label "my counter" -y --keyring-backend test`



# sSCRT Staking Contract
# Overseer Contract
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