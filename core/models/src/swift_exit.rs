

pub struct SwiftExit {
    pub block_number: BlockNumber,
    pub operation_id: OperationId,
    pub account_id: u32,
    pub token_id: u16,
    pub token_amount: BigDecimal,
    pub fee: BigDecimal,
    pub swift_exit_fee: BigDecimal,
    pub owner: Address,
    pub recipient: Address,
    pub supply_amount: BigDecimal,
    pub aggr_signature: Option<Signature>,
    pub signers_bitmask: u16
}

impl SwiftExit {

    const LENGTH: u8 = 109;

    pub fn save_new_request(&mut self) -> Result<(), Error> {
        let supply_amount = price_calculator::calc_supply_amount(self.token_id, self.token_amount);
        self.supply_amount = supply_amount;
        storage.save_swift_exit(&self)?;
        Ok(())
    }

    pub fn into_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(self.block_number.as_bytes());
        out.extend_from_slice(self.operation_id.as_bytes());
        out.extend_from_slice(self.account_id.as_bytes());
        out.extend_from_slice(self.token_id.as_bytes());
        out.extend_from_slice(self.token_amount.as_bytes());
        out.extend_from_slice(self.fee.as_bytes());
        out.extend_from_slice(self.swift_exit_fee.as_bytes());
        out.extend_from_slice(self.owner.get_bytes());
        out.extend_from_slice(self.recipient.get_bytes());
        out.extend_from_slice(self.supply_amount.as_bytes());
        out
    }

    pub fn add_aggr_signature(
        &mut self,
        aggr_signature: Signature,
        signers: Vec<Address>
    ) -> Result<(), Error> {
        let signers_bitmask = governance_interactor.get_bitmask(signers)?;
        self.aggr_signature = aggr_signature;
        self.signers_bitmask = signers_bitmask;
        Ok(())
    }

    pub fn load_from_storage(
        block_number: BLockNumber,
        operation_id: OperationId
    ) -> Option<Self> {
        storage.load_swift_exit(
            block_number,
            operation_id
        ).ok()?;
    }

    pub fn commit_swift_exit(&self) -> Result<(), Error> {
        if self.aggr_signature.is_none() {
            return Err()
        }
        if self.signers_bitmask == 0 {
            return Err()
        }
        let bytes = self.into_bytes();
        request_sender.send_swift_exit_to_contract(bytes, self.aggr_signature, self.signers_bitmask)?;
        Ok(())
    }
}





