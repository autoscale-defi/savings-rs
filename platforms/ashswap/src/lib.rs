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

    /// Processes a deposit into the farming mechanism.
    ///
    /// This function serves as an entry point for the controller to deposit funds that are intended for farming.
    /// It expects:
    /// - Only one ESDT transfer at a time.
    /// - The transferred token must match the `asset_token_identifier` (commonly USDC).
    ///
    /// Instead of directly depositing the funds into farms, this endpoint leverages the Autoscale's zap mechanism.
    /// This allows for the efficient conversion of the deposited assets into LP (Liquidity Provider) tokens,
    /// which are then utilized for farming.
    ///
    /// # Note
    /// Ensure the deposited token type matches the expected `asset_token_identifier`.
    ///
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
            // Let's compute how much tokens from the payment we need to send to the farms
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

            // Let's convert the asset token payment into the LP token that will be deposited into the farm
            let new_lp_payment = self.zap_in_payment(
                &pool.pool_address,
                asset_payment
            );


            require!(
                new_lp_payment.amount > 0,
                "no lp token returned"
            );

            // If any existing farm position exists, we want to retrieve it in order to merge it with the future new position
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
            // Since we may have to merge an old position, we might receive additional tokens representing rewards
            // We don't want to handle them here to avoid huge gas costs in the transaction
            // So we keep them apart to handle them later in the "claimRewards" endpoint
            for other_payment in enter_farms_payments.other_payments.iter() {
                self.waiting_rewards().push(&other_payment);
            }
        }

        // TEMP : This is only to be able to deliver a quick PoC for the hackathon.
        self.deposited_assets().update(|amount| *amount += payment.amount);
    }

    /// Processes a withdrawal request from the farming mechanism.
    ///
    /// The controller initiates this function to withdraw funds from the farms. It can request any withdrawal
    /// amount, provided this contract holds sufficient assets to fulfill the request. It's crucial to understand that
    /// the actual responsibility for validating the withdrawal amount resides with the controller contract, not with
    /// this function.
    ///
    /// # Parameters
    /// - `amount`: The quantity the controller wishes to withdraw. This does not include potential rewards. Consequently,
    /// the actual funds received by the controller may exceed the specified amount due to accumulated rewards.
    ///
    /// # Returns
    /// A `ManagedVec` containing `EsdtTokenPayment` objects that represent all the payments dispatched to the controller.
    /// If there are any unknown tokens (i.e., rewards that cannot be converted to the main asset token), they are
    /// transferred directly without any conversion.
    ///
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

            // This should not append but since this is only a proof of concept, some technical aspects are subject to change
            // Such as when we can add/remove farms, so current_farm_position_mapper might be empty
            // In the future we may forbid to add/remove farms after the contract deployment
            if current_farm_position_mapper.is_empty() {
                continue
            }

            let current_farm_position = current_farm_position_mapper.get();

            // We want to withdraw the same % in each farm
            // No need to use the weight here, it has been used already in the deposit endpoint
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

            // Let's get back LP tokens
            let exit_farm_result = self.exit_farm_forward(
                position_to_exit,
                &self.lp_token_identifier_for_pool(&pool.pool_address).get(),
                &pool.farm_address
            );

            let asset_token_identifier = self.asset_token_identifier().get();

            // Let's convert LP to asset token
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

    /// Processes the claim for pending rewards initiated by the controller.
    ///
    /// This function aggregates and claims not only rewards pending in the farms but also those that have
    /// accumulated post execution of the deposit endpoint. Essentially, it ensures the controller gets
    /// both active and latent rewards.
    ///
    /// # Returns
    /// A `ManagedVec` containing `EsdtTokenPayment` objects that represent all the rewards dispatched to the controller.
    /// If there are any unrecognized tokens (i.e., rewards that cannot be converted to the main asset token),
    /// they are transferred directly without any conversion.
    ///
    #[endpoint(claimRewards)]
    fn claim_rewards_endpoint(&self) -> ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>> {
        let caller = self.blockchain().get_caller();

        require!(
            caller == self.controller_address().get(),
            "Only the controller can call this endpoint"
        );

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

        self.handle_rewards(&caller, &all_rewards)
    }

    /// Processes and distributes rewards represented as an array of payments.
    ///
    /// This function goes through each reward and attempts to convert it to the main asset token. If a
    /// reward cannot be converted, it is sent directly without any changes.
    ///
    /// # Parameters
    /// - `receiver`: The address intended to receive the processed rewards.
    /// - `rewards`: A collection of `EsdtTokenPayment` objects representing the rewards to be processed.
    ///
    /// # Returns
    /// A `ManagedVec` containing `EsdtTokenPayment` objects representing the processed rewards ready for
    /// distribution to the `receiver`.
    ///
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

    /// Retrieves the total assets currently deposited that are available for withdrawal.
    ///
    /// This view function aids the controller in determining the amount of assets that can be
    /// withdrawn at any given time.
    ///
    /// # Returns
    /// A `BigUint` value representing the total assets deposited and available for withdrawal.
    ///
    /// # Note
    /// This is a temporary view specifically for the hackathon to demonstrate the proof of concept.
    /// In future iterations, this static retrieval will be replaced by a more dynamic computation mechanism.
    ///
    #[view(getDepositedAssets)]
    fn get_deposited_assets(&self) -> BigUint {
        self.deposited_assets().get()
    }

}
