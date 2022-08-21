use cosmwasm_std::StdResult;
use cosmwasm_std::{Decimal, Uint128};
const DECIMAL_FRACTIONAL: Uint128 = Uint128(1_000_000_000u128);

pub fn substraction(nominator: Decimal, denominator: Decimal) -> StdResult<Decimal> {
    Ok(Decimal::from_ratio(
        (nominator * DECIMAL_FRACTIONAL - denominator * DECIMAL_FRACTIONAL)?,
        DECIMAL_FRACTIONAL,
    ))
}

pub fn multiply(nominator: Decimal, denominator: Decimal) -> Decimal {
    Decimal::from_ratio(
        nominator * DECIMAL_FRACTIONAL * denominator,
        DECIMAL_FRACTIONAL,
    )
}

pub fn decimal_multiplication(a: Decimal, b: Decimal) -> Decimal {
    Decimal::from_ratio(a * DECIMAL_FRACTIONAL * b, DECIMAL_FRACTIONAL)
}

pub fn calculate_and_print_price(nominator: Uint128, denominator: Uint128, index: usize) -> StdResult<String> {
    let result =  Decimal::from_ratio(
        nominator.0,
        denominator.0,
    );
    return Ok(result.to_string())
}

pub fn convert_uint128_to_decimal(val: Uint128) -> StdResult<Decimal>{
    let result: Decimal = Decimal::from_ratio(val.0, Uint128(1).0);
    Ok(result)
}