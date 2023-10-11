multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct SavingsTokenAttributes<M: ManagedTypeApi> {
    pub reward_per_share: BigUint<M>,
    pub accumulated_rewards: BigUint<M>,
    pub last_bloc: u64
}

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct UnbondTokenAttributes {
    pub unlock_epoch: u64
}

#[multiversx_sc::module]
pub trait TokenModule {
    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(registerSavingsToken)]
    fn register_savings_token(
        &self,
        token_display_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        num_decimals: usize,
    ) {
        let issue_cost = self.call_value().egld_value();
        self.savings_token().issue_and_set_all_roles(
            EsdtTokenType::Meta,
            issue_cost.clone_value(),
            token_display_name,
            token_ticker,
            num_decimals,
            None,
        );
    }

    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(registerUnbondToken)]
    fn register_unbond_token(
        &self,
        token_display_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        num_decimals: usize,
    ) {
        let issue_cost = self.call_value().egld_value();
        self.unbond_token().issue_and_set_all_roles(
            EsdtTokenType::Meta,
            issue_cost.clone_value(),
            token_display_name,
            token_ticker,
            num_decimals,
            None,
        );
    }

    #[storage_mapper("savingsTokenId")]
    fn savings_token(&self) -> NonFungibleTokenMapper<Self::Api>;

    #[storage_mapper("unbondToken")]
    fn unbond_token(&self) -> NonFungibleTokenMapper<Self::Api>;
}