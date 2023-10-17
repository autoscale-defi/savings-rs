multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait PlatformModule {
    fn invest_in_platform(&self, sc_address: ManagedAddress, amount: BigUint) {
        let payment = EsdtTokenPayment::new(self.usdc_token().get_token_id(), 0, amount);

        self.platform_proxy(sc_address)
            .deposit()
            .with_esdt_transfer(payment)
            .execute_on_dest_context::<IgnoreValue>();
    }

    fn withdraw_from_platform(&self, sc_address: ManagedAddress, amount: BigUint) -> ManagedVec<EsdtTokenPayment> {
        let payment = EsdtTokenPayment::new(self.usdc_token().get_token_id(), 0, amount.clone());

        self.platform_proxy(sc_address)
            .withdraw(amount)
            .with_esdt_transfer(payment)
            .execute_on_dest_context()
    }

    fn claim_rewards_for_platform(&self, sc_address: ManagedAddress) -> ManagedVec<EsdtTokenPayment> {
        self.platform_proxy(sc_address)
            .claim_rewards_endpoint()
            .execute_on_dest_context()
    }

    fn get_total_deposited_for_platform(&self, sc_address: ManagedAddress) -> BigUint {
        self.platform_proxy(sc_address)
            .get_deposited_assets()
            .execute_on_dest_context()
    }

    #[proxy]
    fn platform_proxy(&self, sc_address: ManagedAddress) -> platform::Proxy<Self::Api>;

    #[storage_mapper("usdcTokenId")]
    fn usdc_token(&self) -> FungibleTokenMapper<Self::Api>;
}
