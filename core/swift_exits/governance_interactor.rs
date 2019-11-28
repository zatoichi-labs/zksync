pub fn is_validator_active(addr: Address) -> Result<Bool, Error> {

}

pub fn get_validator_supply(addr: Address) -> Result<BigDecimal, Error> {

}

pub fn get_validator_id(addr: Address) -> Result<u16, Error> {

}

pub fn get_validator_pubkey(addr: Address) -> Result<Pubkey, Error> {

}

pub fn get_bitmask(validators: Vec<Address>) -> Result<u16, Error> {
    let mut bitmask = [u8; 0xFFFF];
    for addr in validators {
        let id = get_validator_id(addr)?;
        if is_validator_active(addr)? && get_validator_supply(addr)? > 0 {
            bitmask[id] = 1;
        }
    }
    Ok(bitmask.into_u16())
}