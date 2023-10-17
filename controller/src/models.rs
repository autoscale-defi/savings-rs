multiversx_sc::derive_imports!();
multiversx_sc::imports!();

/// A Struct to export all the parameters of the controller off-chain
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

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi)]
pub struct PlatformInfo<M: ManagedTypeApi> {
    pub name: ManagedBuffer<M>,
    pub sc_address: ManagedAddress<M>,
    pub weight: u64,
}

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct SavingsTokenAttributes<M: ManagedTypeApi> {
    pub initial_rewards_per_share: BigUint<M>,
    pub accumulated_rewards: BigUint<M>,
    pub total_shares: BigUint<M>,
}

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct UnbondTokenAttributes {
    pub unlock_epoch: u64,
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, Clone)]
pub enum Phase {
    Accumulation,
    Depletion,
}
