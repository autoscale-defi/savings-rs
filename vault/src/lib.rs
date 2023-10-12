#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait VaultContract {
    #[init]
    fn init(&self, usdc_token_id: TokenIdentifier, controller_addr: ManagedAddress) {
        self.usdc_token_id().set_if_empty(usdc_token_id);
        self.controller_addr().set_if_empty(&controller_addr);
    }

    #[payable("*")]
    #[endpoint(increaseReserve)]
    fn increase_reserve(&self) {
        self.require_caller_is_controller();

        let payments = self.call_value().all_esdt_transfers();
        self.usdc_token_id().require_all_same_token(&payments);

        let mut new_rewards = BigUint::zero();
        for payment in payments.iter() {
            new_rewards += payment.amount;
        }
        self.rewards_reserve().update(|r| *r += new_rewards);
    }

    #[endpoint(sendRewards)]
    fn send_rewards(&self, destination: ManagedAddress, amount: BigUint) {
        self.require_caller_is_controller();

        self.send().direct_esdt(
            &destination,
            &self.usdc_token_id().get_token_id(),
            0,
            &amount,
        );
        self.rewards_reserve().update(|r| *r -= amount);
    }

    fn require_caller_is_controller(&self) {
        let caller = self.blockchain().get_caller();
        let controller_addr = self.controller_addr().get();
        require!(
            caller == controller_addr,
            "Only the controller can call this function"
        );
    }

    #[view(getControllerAddress)]
    #[storage_mapper("controllerAddr")]
    fn controller_addr(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("usdcTokenId")]
    fn usdc_token_id(&self) -> FungibleTokenMapper<Self::Api>;

    #[view(getRewardsReserve)]
    #[storage_mapper("rewardsReserve")]
    fn rewards_reserve(&self) -> SingleValueMapper<BigUint>;
}
