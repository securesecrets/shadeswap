# Shadeswap Formulas

## Calculating Swap Amount
### References
__Token_0:__ Base token (e.g. sSHD for the pair sSHD/sSCRT)
__Token_1:__ Quote token (e.g. sSCRT for the pair sSHD/sSCRT)
__Token_0_Pool:__ Total amount in the token pool of the base token
__Token_1_Pool:__ Total amount in the token pool of the quote token
__Slippage:__ Fraction as a decimal number (e.g. 0.02 for 2% slippage)
### Buy
__Amount_In:__ Amount of Token_0
__Amount_Out:__ Amount of Token_1
__Target_Token:__ Token_1
__Source_Token:__ Token_0
__Target_Token_Pool:__ Total amount in the token pool of the Target_Token
__Source_Token_Pool:__ Total amount in the token pool of the Source_Token
### Sell
__Amount_In:__ Amount of Token_1
__Amount_Out:__ Amount of Token_0
__Target_Token:__ Token_0
__Source_Token:__ Token_1
__Target_Token_Pool:__ Total amount in the token pool of the Target_Token
__Source_Token_Pool:__ Total amount in the token pool of the Source_Token
### 1. Calculation of fee
`Fee = Amount_In * Fee.Nom / Fee.Denom`
### 2. Calculation of Amount_Out
`Amount_Out = Target_Token_Pool * (Amount_In - Fee) / (Source_Token_Pool + Amount_In - Fee)`
### 3. Calculation of price
`Price = Amount_Out / (Amount_In - Fee)`

## Calculating Liquidity
### References
__Deposit_0:__ The amount of deposited Token_0
__Deposit_1:__ The amount of deposited Token_1
__Slippage:__ Fraction as a decimal number (e.g. 0.02 for 2%)
__Slippage_amount:__ Accepting slippage
__LP Token:__ Liquidity token
__Withdraw_amount:__ Amount of LP Token to remove from liquidity
__Slippage:__ Fraction as a decimal number (e.g. 0.02 for 2%)
__Pool_withdraw_0:__ Amount of Token_0 to receive
__Pool_withdraw_1:__ Amount of Token_1 to receive
__Min:__ Function which returns the minimum of the two input
### 1. Assert slippage acceptance for adding liquidity
__IF__ `Slippage * Deposit_0 / Deposit_1 > Token_0_pool / Token_1_pool`
__THEN__ `Slippage_amount = 1 - Slippage`
__OTHERWISE__ `Thow an exception and don't allow to proceed.`
### 2. Add liquidity calculation
__IF__ `LP balance is 0`
__THEN__ `sqrt(Deposit_0 * Deposit_1)`
__OTHERWISE__ `min(Deposit_0 * LP_token_balance / Token_0_pool, Deposit_1 * LP_token_balance / Token_1_pool)`
### 3. Remove liquidity calculation
`Pool_withdraw_0 = Token_0_pool * Withdraw_amount / LP_token_balance`
`Pool_withdraw_1 = Token_1_pool * Withdraw_amount / LP_token_balance`

## Staking
### References
__Current_timestamp:__ the timestamp of the block in milliseconds where the claim reward or adding new staker is called.
__Last_timestamp:__ the last stored timestamp of the block in milliseconds where claim reward method was called in staking contract.
__Staker_percentage:__ % from whole total staking amount belong to staker
__Staker:__ Person who is staking LP token
__Stake_amount:__ Amount of LP token which belongs to staker
__Total_staking_amount:__ total amount of lp token of all stakers
__Seconds:__ 24 * 60 * 60 * 1000 = 86,400,000
__Daily_reward_amount:__ Staking amount which is constant and set by admin
__Cons:__ Constant value used for calculating Staker_percentage, which is `100`
### 1. Staker_percentage (%)
`Staker_percentage = Stake_amount * Cons / Total_staking_amount`
### 2. Claim reward calculation
__IF__ `last_timestamp < current_timestamp`
__THEN__ `0`
__OTHERWISE__ `Daily_reward_amount * Staker_percentage * (current_timestamp - last_timestamp) / (Seconds * Cons)`
