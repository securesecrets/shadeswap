use cosmwasm_std::StdResult;
use cosmwasm_std::{Decimal, Uint128};

pub fn substraction(nominator: Decimal, denominator: Decimal) -> StdResult<Decimal> {
    let DECIMAL_FRACTIONAL: Uint128 = Uint128::from(1_000_000_000u128);
    Ok(Decimal::from_ratio(
        (nominator * DECIMAL_FRACTIONAL - denominator * DECIMAL_FRACTIONAL),
        DECIMAL_FRACTIONAL,
    ))
}

pub fn multiply(nominator: Decimal, denominator: Decimal) -> Decimal {
    let DECIMAL_FRACTIONAL: Uint128 = Uint128::from(1_000_000_000u128);
    Decimal::from_ratio(
        nominator * DECIMAL_FRACTIONAL * denominator,
        DECIMAL_FRACTIONAL,
    )
}

pub fn decimal_multiplication(a: Decimal, b: Decimal) -> Decimal {
    let DECIMAL_FRACTIONAL: Uint128 = Uint128::from(1_000_000_000u128);
    Decimal::from_ratio(a * DECIMAL_FRACTIONAL * b, DECIMAL_FRACTIONAL)
}

pub fn calculate_and_print_price(nominator: Uint128, denominator: Uint128, index: usize) -> StdResult<String> {
    let result =  Decimal::from_ratio(
        nominator,
        denominator,
    );
    return Ok(result.to_string())
}