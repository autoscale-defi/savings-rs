#![no_std]

mod zap_proxy;
mod storage;
mod holder_proxy;
mod admin;
mod pool_proxy;
mod farm_proxy;

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait AshSwapPlatformContract: ContractBase
    + storage::StorageModule + admin::AdminModule + zap_proxy::ZapProxyModule + holder_proxy::HolderProxyModule + pool_proxy::PoolProxyModule + farm_proxy::FarmProxyModule
{
    #[init]
    fn init(
        &self,
        controller_address: &ManagedAddress<Self::Api>,
        holder_address: &ManagedAddress<Self::Api>,
        zap_address: &ManagedAddress<Self::Api>,
        asset_token_identifier: &TokenIdentifier<Self::Api>,
    ) {
        self.controller_address().set_if_empty(controller_address);
        self.holder_address().set_if_empty(holder_address);
        self.zap_address().set_if_empty(zap_address);
        self.asset_token_identifier().set_if_empty(asset_token_identifier);
    }

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
                let amount = &payment.amount * &BigUint::from(pool.weight) / &BigUint::from(total_weight);
                left_payment_amount -= &amount;

                amount
            };

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

        // TEMP : This is only to be able to deliver a quick PoC for the hackathon.
        self.deposited_assets().update(|amount| *amount += payment.amount);
    }

    #[endpoint]
    fn withdraw(&self, amount: BigUint<Self::Api>) -> ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>> {
        let caller = self.blockchain().get_caller();

        require!(
            caller == self.controller_address().get(),
            "Only the controller can call this endpoint"
        );

        let total_deposited_amount = self.deposited_assets().get();

        require!(
            amount <= total_deposited_amount,
            "Too much withdrawal amount requested"
        );

        let pools_mapper = self.pools();
        let pools = pools_mapper.values();

        let mut results = ManagedVec::new();
        for pool in pools {
            let current_farm_position_mapper = self.current_position_for_farm(&pool.farm_address);
            let current_farm_position = current_farm_position_mapper.get();
            let amount_to_exit: BigUint<Self::Api> = &current_farm_position.amount * &amount / &total_deposited_amount;
            let position_to_exit = EsdtTokenPayment::new(
                current_farm_position.token_identifier.clone(),
                current_farm_position.token_nonce,
                amount_to_exit
            );

            if position_to_exit.amount == current_farm_position.amount {
                current_farm_position_mapper.clear();
            } else {
                current_farm_position_mapper.update(|position| position.amount -= &position_to_exit.amount);
            }

            let exit_farm_result = self.exit_farm_forward(
                position_to_exit,
                &self.lp_token_identifier_for_pool(&pool.pool_address).get(),
                &pool.farm_address
            );

            let asset_token_identifier = self.asset_token_identifier().get();
            let zap_out_result = self.zap_out_payment(
                &pool.pool_address,
                &asset_token_identifier,
                exit_farm_result.lp_token_payment
            );
            if zap_out_result.amount > 0 {
                self.send()
                    .direct_esdt(
                        &caller,
                        &zap_out_result.token_identifier,
                        zap_out_result.token_nonce,
                        &zap_out_result.amount
                    );
                results.push(zap_out_result);
            }

            let sent_rewards = self.handle_rewards(&caller, &exit_farm_result.other_payments);
            results.append_vec(sent_rewards);
        }

        // TEMP : This is only to be able to deliver a quick PoC for the hackathon.
        self.deposited_assets().update(|deposited_amount| *deposited_amount -= &amount);

        results
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

            let claim_rewards_result = self.claim_farm_rewards(
                current_farm_position_mapper.get(),
                &pool.farm_address
            );

            current_farm_position_mapper.set(claim_rewards_result.share_token_payment);

            for reward in claim_rewards_result.other_payments.iter() {
                if reward.amount > 0 {
                    all_rewards.push(reward)
                }
            }
        }

        self.handle_rewards(&receiver, &all_rewards)
    }

    fn handle_rewards(
        &self,
        receiver: &ManagedAddress<Self::Api>,
        rewards: &ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>>
    ) -> ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>> {
        // Let's swap rewards to the asset token
        let mut total_assets_payment = EsdtTokenPayment::new(
            self.asset_token_identifier().get(),
            0,
            BigUint::zero()
        );
        let mut results = ManagedVec::new();
        for reward in rewards.iter() {
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
