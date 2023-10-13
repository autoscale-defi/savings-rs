use crate::phase::Phase;

multiversx_sc::derive_imports!();
multiversx_sc::imports!();

#[derive(TopEncode, TypeAbi)]
pub struct ControllerParametersDTO<M: ManagedTypeApi> {
    pub phase: Phase,
    pub min_unbond_epochs: u64,
    pub force_withdraw_fees_percentage: u64,
    pub deposit_fees_percentage: u64,
    pub rewards_per_share_per_block: BigUint<M>,
    pub usdc_token_id: TokenIdentifier<M>,
    pub savings_token_id: TokenIdentifier<M>,
    pub unbond_token_id: TokenIdentifier<M>,
}
