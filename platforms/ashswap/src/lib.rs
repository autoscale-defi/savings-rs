#![no_std]

mod zap_proxy;
mod storage;
mod holder_proxy;

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait AshSwapPlatformContract: ContractBase
    + storage::StorageModule + zap_proxy::ZapProxyModule + holder_proxy::HolderProxyModule
{
    #[endpoint]
    #[payable("*")]
    fn deposit(&self) {
        let caller = self.blockchain().get_caller();

        require!(
            caller == self.controller_address().get(),
            "Only the controller can call this endpoint"
        );

        let payment = self.call_value().single_esdt();
        let asset_token_identifier = self.asset_token_identifier().get();

        require!(
            payment.token_identifier == asset_token_identifier,
            "Wrong token payment"
        );

        let pools_mapper = self.pools();
        let pools = pools_mapper.values();
        let total_weight = self.pools_total_weight().get();
        let mut left_payment_amount = payment.amount.clone();
        let mut used_weight = 0u64; // no .len() on pools, this is an alternative way to know if the pool in the for loop is the last one

        for pool in pools {
            let payment_amount = if used_weight + pool.weight == total_weight { // this is the last pool, let's use the whole unused payment
                core::mem::take(&mut left_payment_amount)
            } else {
                &payment.amount * &BigUint::from(pool.weight) / &BigUint::from(total_weight)
            };

            left_payment_amount -= &payment_amount;
            used_weight += pool.weight;

            let asset_payment = EsdtTokenPayment::new(
                payment.token_identifier.clone(),
                payment.token_nonce,
                payment_amount
            );

            let new_lp_payment = self.zap_in_payment(
                &pool.pool_address,
                asset_payment
            );

            require!(
                new_lp_payment.amount > 0,
                "no lp token returned"
            );

            let current_farm_position_mapper = self.current_position_for_farm(&pool.farm_address);
            let opt_current_farm_position_payment = if current_farm_position_mapper.is_empty() {
                None
            } else {
                Some(current_farm_position_mapper.get())
            };

            let enter_farms_payments = self.enter_farm(
                new_lp_payment,
                opt_current_farm_position_payment,
                &pool.farm_address
            );

            current_farm_position_mapper.set(enter_farms_payments.share_token_payment);
            for other_payment in enter_farms_payments.other_payments.iter() {
                self.waiting_rewards().push(&other_payment);
            }
        }
    }
}
