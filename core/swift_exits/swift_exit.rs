

pub struct SwiftExit {
    pub block_number: BlockNumber,
    pub operation_id: OperationId,
    pub withdraw_op: Withdraw,
    pub aggr_signature: Signature,
    pub signers_bitmask: U256
}

impl SwiftExit {

    const LENGTH: u8 = 109;

    pub fn submit_swift_exit(
        block_number: BlockNumber,
        operation_id: OperationId,
        withdraw_op: &Withdraw
    ) -> Self {

    }

    pub fn into_bytes(&self) -> Vec<u8> {

    }

    pub fn add_aggr_signature(&mut self, aggr_signature: Signature, signers_bitmask: U256) {

    }

    pub fn send_to_contract(&self) {

    }

}





