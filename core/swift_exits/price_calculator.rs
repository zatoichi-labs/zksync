pub fn calc_supply_amount(token_id: TokenId, amount: BigDecimal) -> Result<BigDecimal, Error> {
    let matter_token_id = governance.matter_token_id()?;
    if matter_token_id == token_id {
        return Ok(amount)
    }

    let matter_token_address = governance.validate_token_id(matter_token_id)?;
    let c_matter_token_address = governance.get_c_token_address(matter_token_address)?;

    let supply_token_address = governance.validate_token_id(token_id)?;
    let c_supply_token_address = governance.get_c_token_address(supply_token_address)?;

    let matter_token_price = price_oracle.get_underlying_price(c_matter_token_address)?;
    let supply_token_price = price_oracle.get_underlying_price(c_supply_token_address)?;

    let matter_token_collateral_factor = comptroller.collateral_factor(c_matter_token_address)?;

    Ok(amount * matter_token_price / (supply_token_price * collateral_factor * SUPPLY_COEFF))
}