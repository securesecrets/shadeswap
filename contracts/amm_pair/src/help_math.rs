use shadeswap_shared::fadroma::scrt::{Decimal, StdResult, Uint128};

const DECIMAL_FRACTIONAL: Uint128 = Uint128(1_000_000_000u128);

pub fn substraction(nominator: Decimal, denominator: Decimal) -> StdResult<Decimal> {
    Ok(Decimal::from_ratio(
        (nominator * DECIMAL_FRACTIONAL - denominator * DECIMAL_FRACTIONAL)?,
        DECIMAL_FRACTIONAL,
    ))
}

pub fn multiply(nominator: Decimal, denominator: Decimal) -> Decimal {
    Decimal::from_ratio(
        (nominator * DECIMAL_FRACTIONAL * denominator),
        DECIMAL_FRACTIONAL,
    )
}

pub fn decimal_multiplication(a: Decimal, b: Decimal) -> Decimal {
    Decimal::from_ratio(a * DECIMAL_FRACTIONAL * b, DECIMAL_FRACTIONAL)
}


pub fn calculate_and_print_price(nominator: u128, denominator: u128, index: usize) -> StdResult<String> {
    let mut price_to_string = "".to_string();
    if index == 0 {
        let result =  Decimal::from_ratio(
            nominator,
            denominator,
        );
        return Ok(result.to_string())
    } // SELL
    
    if index == 1 {
        let result =  Decimal::from_ratio(nominator,denominator);
        let temp = Decimal::one();
        let final_result = decimal_multiplication(result, temp);
       return Ok(final_result.to_string())
    }
    Ok("".to_string())   
}
