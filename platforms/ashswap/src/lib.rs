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

    #[endpoint(claimRewards)]
    fn claim_rewards_endpoint(&self) -> ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>> {
        let caller = self.blockchain().get_caller();

        require!(
            caller == self.controller_address().get(),
            "Only the controller can call this endpoint"
        );

        self.claim_all_rewards(&caller)
    }

    fn claim_all_rewards(&self, receiver: &ManagedAddress<Self::Api>) -> ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>> {
        let pools_mapper = self.pools();
        let pools = pools_mapper.values();

        let mut all_rewards: ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>> = ManagedVec::new();
        for waiting_rewards in self.waiting_rewards().iter() {
            all_rewards.push(waiting_rewards)
        }

        for pool in pools {
            let current_farm_position_mapper = self.current_position_for_farm(&pool.farm_address);
            if current_farm_position_mapper.is_empty() {
                continue
            }

            let rewards = self.claim_farm_rewards(
                current_farm_position_mapper.get(),
                &pool.farm_address
            );

            for reward in rewards.iter() {
                if reward.amount > 0 {
                    all_rewards.push(reward)
                }
            }
        }

        // Let's swap rewards to the asset token
        let mut total_assets_payment = EsdtTokenPayment::new(
            self.asset_token_identifier().get(),
            0,
            BigUint::zero()
        );
        let mut results = ManagedVec::new();
        for reward in all_rewards.iter() {
            if reward.token_identifier == total_assets_payment.token_identifier {
                total_assets_payment.amount += &reward.amount;
                continue
            }

            if self.swappable_tokens().contains(&reward.token_identifier) {
                total_assets_payment.amount += self.swap_payment(
                    reward,
                    &total_assets_payment.token_identifier
                ).amount;
            } else {
                results.push(reward)
            }
        }
        results.push(total_assets_payment);

        for result in results.iter() {
            if result.amount > 0 {
                self.send()
                    .direct_esdt(
                        receiver,
                        &result.token_identifier,
                        result.token_nonce,
                        &result.amount
                    );
            }
        }

        results
    }

}
