#![no_std]
multiversx_sc::imports!();

use models::{
    ClaimRewardsResultType, ControllerParametersDTO, Phase, SavingsTokenAttributes,
    UnbondTokenAttributes, WithdrawResultType,
};
use multiversx_sc_modules::default_issue_callbacks;

pub mod config;
pub mod models;
pub mod proxy;
pub mod rewards;
pub mod token;

const PERCENTAGE_DIVIDER: u64 = 10000;

#[multiversx_sc::contract]
pub trait ControllerContract:
    token::TokenModule
    + rewards::RewardsModule
    + config::ConfigModule
    + proxy::ProxyModule
    + default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[init]
    fn init(
        &self,
        usdc_token_id: TokenIdentifier,
        phase: Phase,
        min_unbond_epochs: u64,
        deposit_fees: u64,
        performance_fees: u64,
        withdraw_fees_perc: u64,
    ) {
        self.usdc_token().set_if_empty(usdc_token_id);
        self.phase().set_if_empty(phase.clone());
        self.min_unbond_epochs().set(min_unbond_epochs);
        self.deposit_fees_percentage(phase).set(deposit_fees);
        self.performance_fees().set(performance_fees);
        self.force_withdraw_fees_percentage()
            .set(withdraw_fees_perc);

        self.last_update_for_min_liq_reserve_needed()
            .set_if_empty(self.blockchain().get_block_epoch());
    }

    /// User deposits USDC and receives savings tokens
    #[payable("*")]
    #[endpoint]
    fn deposit(&self) -> EsdtTokenPayment<Self::Api> {
        let payments = self.call_value().all_esdt_transfers();

        let usdc_payment = payments
            .try_get(0)
            .unwrap_or_else(|| sc_panic!("empty payments"));
        let additional_payments = payments.slice(1, payments.len()).unwrap_or_default();

        self.usdc_token()
            .require_same_token(&usdc_payment.token_identifier);
        self.savings_token()
            .require_all_same_token(&additional_payments);

        let phase = self.get_phase();
        let usdc_amount_to_deposit = self.charge_and_send_deposit_fees(phase, &usdc_payment.amount);

        let new_savings_token =
            self.create_savings_token_by_merging(&usdc_amount_to_deposit, &additional_payments);

        self.liquidity_reserve()
            .update(|x| *x += usdc_amount_to_deposit);

        let caller = self.blockchain().get_caller();
        self.send()
            .direct_non_zero_esdt_payment(&caller, &new_savings_token);

        new_savings_token
    }

    /// User withdraws unbond tokens from savings tokens, or directly USDC by paying a fee
    /// withdraw also triggers the claimRewards function
    #[payable("*")]
    #[endpoint]
    fn withdraw(&self, opt_force_withdraw: OptionalValue<bool>) -> WithdrawResultType<Self::Api> {
        let payments = self.call_value().all_esdt_transfers();
        self.savings_token().require_all_same_token(&payments);

        let rewards = self.merge_savings_tokens(&payments);
        require!(rewards.total_shares > 0, "Payment amount cannot be zero");

        let current_epoch = self.blockchain().get_block_epoch();
        let min_unbond_epochs = self.min_unbond_epochs().get();

        let force_withdraw = opt_force_withdraw.into_option().unwrap_or(false);

        let output_payment = if force_withdraw {
            let fees_percentage = self.force_withdraw_fees_percentage().get();
            let fees_amount = rewards.total_shares.clone() * fees_percentage / PERCENTAGE_DIVIDER;
            let net_amount = &rewards.total_shares - &fees_amount;

            self.send_fees(&fees_amount);

            EsdtTokenPayment::new(self.usdc_token().get_token_id(), 0, net_amount)
        } else {
            let unbond_token_attr = UnbondTokenAttributes {
                unlock_epoch: current_epoch + min_unbond_epochs,
            };
            let unbond_token_payment = self
                .unbond_token()
                .nft_create(rewards.total_shares.clone(), &unbond_token_attr);

            self.liquidity_needed_for_epoch(unbond_token_attr.unlock_epoch)
                .update(|x| *x += rewards.total_shares.clone());

            unbond_token_payment
        };

        // user wants to withdraw so even if the real amount is 0 (rounded), the tx goes through and positions are closed
        let rewards_payment = self.send_rewards(
            self.blockchain().get_caller(),
            self.get_real_usdc_rewards_amount(&rewards.accumulated_rewards),
        );
        self.burn_savings_tokens(&payments);

        let caller = self.blockchain().get_caller();
        self.send()
            .direct_non_zero_esdt_payment(&caller, &output_payment);

        (output_payment, rewards_payment).into()
    }

    /// User send unbond tokens to the contract and receives USDC if the unlock epoch is reached
    /// In the future, this endpoint should accept a list of unbond tokens
    #[payable("*")]
    #[endpoint]
    fn unbond(&self) -> EsdtTokenPayment {
        let payment = self.call_value().single_esdt();
        self.unbond_token()
            .require_same_token(&payment.token_identifier);
        require!(payment.amount > 0, "Payment amount cannot be zero");

        self.update_min_liq_reserve_needed();

        let unbond_token_attr: UnbondTokenAttributes = self
            .unbond_token()
            .get_token_attributes(payment.token_nonce);
        require!(
            self.blockchain().get_block_epoch() >= unbond_token_attr.unlock_epoch,
            "Cannot unbond before unlock epoch"
        );
        require!(
            self.liquidity_reserve().get() >= payment.amount,
            "Not enough liquidity"
        );

        self.unbond_token()
            .nft_burn(payment.token_nonce, &payment.amount);
        self.liquidity_reserve()
            .update(|x| *x -= payment.amount.clone());

        let output_payment =
            EsdtTokenPayment::new(self.usdc_token().get_token_id(), 0, payment.amount.clone());

        self.send()
            .direct_non_zero_esdt_payment(&self.blockchain().get_caller(), &output_payment);

        self.min_liquidity_reserve_needed()
            .update(|x| *x -= payment.amount);

        output_payment
    }

    /// User claims rewards from savings tokens
    /// savings tokens are burned
    /// new savings tokens are minted with the new rewards per share
    #[payable("*")]
    #[endpoint(claimRewards)]
    fn claim_rewards(&self) -> ClaimRewardsResultType<Self::Api> {
        let payments = self.call_value().all_esdt_transfers();
        self.savings_token().require_all_same_token(&payments);

        let rewards = self.merge_savings_tokens(&payments);
        require!(rewards.total_shares > 0, "Payment amount cannot be zero");

        let real_usdc_rewards_amount =
            self.get_real_usdc_rewards_amount(&rewards.accumulated_rewards);
        require!(real_usdc_rewards_amount > 0, "No rewards to claim");

        let new_savings_token_attr = SavingsTokenAttributes {
            initial_rewards_per_share: self.rewards_per_share().get(),
            accumulated_rewards: BigUint::zero(),
            total_shares: rewards.total_shares.clone(),
        };
        let new_savings_token = self
            .savings_token()
            .nft_create(rewards.total_shares.clone(), &new_savings_token_attr);

        self.burn_savings_tokens(&payments);

        let caller = self.blockchain().get_caller();

        let rewards_payment = self.send_rewards(caller.clone(), real_usdc_rewards_amount);
        self.send()
            .direct_non_zero_esdt_payment(&caller, &new_savings_token);

        (new_savings_token, rewards_payment).into()
    }

    /// Admin wallet (or user) can call this endpoint
    /// it claims rewards from all platforms and send them to the vault
    /// Performance fees are taken from the rewards and sent to the fees address
    #[endpoint(claimControllerRewards)]
    fn claim_controller_rewards(&self) {
        let platforms = self.platforms();

        for platform in platforms.iter() {
            let sc_address = platform.sc_address; // will be used to call the platform sc
            let fees_address = self.fees_address().get();

            let claim_rewards_payments = self.claim_rewards_for_platform(sc_address);
            let usdc_token_id = self.usdc_token().get_token_id();

            let mut rewards_payment =
                EsdtTokenPayment::new(usdc_token_id.clone(), 0, BigUint::zero());

            for payment in claim_rewards_payments.iter() {
                if payment.token_identifier == usdc_token_id {
                    rewards_payment.amount += payment.amount;
                } else {
                    self.send().direct_esdt(
                        &fees_address,
                        &payment.token_identifier,
                        0,
                        &payment.amount,
                    );
                }
            }

            let performance_fees = self.performance_fees().get();
            let fees_amount =
                rewards_payment.amount.clone() * performance_fees / PERCENTAGE_DIVIDER;
            self.send_fees(&fees_amount);

            rewards_payment.amount -= fees_amount;
            self.increase_reserve(rewards_payment);
        }
    }

    /// Admin wallet (or user) can call this endpoint
    /// it manages the liquidity reserve
    /// it updates the minimum liquidity reserve needed
    /// it invests or withdraws from the platforms to ensure the minimum liquidity reserve
    #[endpoint(manageLiquidity)]
    fn manage_liquidity(&self) {
        self.update_min_liq_reserve_needed();

        let min_liq_reserve_needed = self.min_liquidity_reserve_needed().get();
        let liquidity_reserve = self.liquidity_reserve().get();
        let liquidity_buffer = self.liquidity_buffer().get();

        let total_liq_needed = min_liq_reserve_needed + liquidity_buffer;

        if liquidity_reserve > total_liq_needed {
            let liquidity_diff = liquidity_reserve - total_liq_needed;
            self.invest(&liquidity_diff);
        } else {
            let liquidity_needed = total_liq_needed - liquidity_reserve;
            self.withdraw_from_platform_contracts(&liquidity_needed);
        }
    }

    // When this function is called, we update the minimum reserved liquidity we need to ensure withdrawals.
    // If the function is not called at every epoch, it loops to update all epochs that have not been updated.
    // As a security, would it be useful to also add the liquidity needed for the current_epoch + 1 to be sure?
    // I think it would be important to do it for at least current_epoch + 2 or 3
    // Otherwise, I think we'll need to withdraw a lot of funds from the investments contracts.
    fn update_min_liq_reserve_needed(&self) {
        let current_epoch: u64 = self.blockchain().get_block_epoch();
        let last_update = self.last_update_for_min_liq_reserve_needed().get();
        let epoch_diff = current_epoch - last_update;

        let mut liquidity = BigUint::zero();

        for epoch in (current_epoch - epoch_diff + 1)..=current_epoch {
            let liq_needed_for_epoch = self.liquidity_needed_for_epoch(epoch).get();
            liquidity += liq_needed_for_epoch;
        }

        self.min_liquidity_reserve_needed()
            .update(|x| *x += liquidity);
        self.last_update_for_min_liq_reserve_needed()
            .set(current_epoch);
    }

    /// This function is called onyl from manage_liquidity endpoint
    /// It invests a given amount in all platforms
    /// If some dust is left, it is invested in the last platform
    fn invest(&self, total_amount: &BigUint) {
        let platforms = self.platforms();
        let total_weight = self.platforms_total_weight().get();

        let mut left_payment_amount = total_amount.clone();
        let mut used_weight = 0;

        for platform in platforms.iter() {
            let invest_amount = if used_weight + platform.weight == total_weight {
                core::mem::take(&mut left_payment_amount)
            } else {
                let calculated_amount =
                    total_amount * &BigUint::from(platform.weight) / &BigUint::from(total_weight);

                left_payment_amount -= calculated_amount.clone();

                calculated_amount
            };
            used_weight += platform.weight;

            let sc_address = platform.sc_address;
            self.invest_in_platform(sc_address, invest_amount);
        }
    }

    /// This function is called only from manage_liquidity endpoint
    /// It withdraws a given amount from all platforms
    fn withdraw_from_platform_contracts(&self, total_amount: &BigUint) {
        let platforms = self.platforms();
        let total_deposited = self.get_total_deposited();

        let mut new_liquidity_amount = BigUint::zero();
        let mut new_rewards = BigUint::zero();

        for platform in platforms.iter() {
            let sc_address = platform.sc_address;

            let amount_deposited = self.get_total_deposited_for_platform(sc_address.clone());
            let amount_to_withdraw = amount_deposited * total_amount / &total_deposited;

            let withdraw_result = self.withdraw_from_platform(sc_address, amount_to_withdraw);

            let withdraw_payment = withdraw_result.get(0);
            let rewards_payment = withdraw_result.get(1);

            new_liquidity_amount += withdraw_payment.amount;
            new_rewards += rewards_payment.amount;
        }
        let rewards_payment =
            EsdtTokenPayment::new(self.usdc_token().get_token_id(), 0, new_rewards);

        self.increase_reserve(rewards_payment);
        self.liquidity_reserve()
            .update(|x| *x += new_liquidity_amount);
    }

    #[view(getTotalDeposited)]
    fn get_total_deposited(&self) -> BigUint {
        let platforms = self.platforms();
        let mut total_deposited = BigUint::zero();

        for platform in platforms.iter() {
            let sc_address = platform.sc_address;
            let amount_deposited = self.get_total_deposited_for_platform(sc_address);

            total_deposited += amount_deposited;
        }

        total_deposited
    }

    fn charge_and_send_deposit_fees(&self, phase: Phase, amount: &BigUint) -> BigUint {
        let fees_percentage = self.deposit_fees_percentage(phase).get();

        if fees_percentage == 0 {
            return amount.clone();
        }

        let fees_amount = amount * fees_percentage / PERCENTAGE_DIVIDER;
        self.send_fees(&fees_amount);

        amount - &fees_amount
    }

    fn send_fees(&self, fees_amount: &BigUint) {
        let fees_address = self.fees_address().get();
        let usdc_token_id = self.usdc_token().get_token_id();

        self.send()
            .direct_esdt(&fees_address, &usdc_token_id, 0, fees_amount);
    }

    #[view(getControllerParameters)]
    fn get_controller_parameters(&self) -> ControllerParametersDTO<Self::Api> {
        let phase = self.get_phase();

        ControllerParametersDTO {
            phase: phase.clone(),
            min_unbond_epochs: self.min_unbond_epochs().get(),
            force_withdraw_fees_percentage: self.force_withdraw_fees_percentage().get(),
            deposit_fees_percentage: self.deposit_fees_percentage(phase).get(),
            rewards_per_share_per_block: self.rewards_per_share_per_block().get(),
            usdc_token_id: self.usdc_token().get_token_id(),
            savings_token_id: self.savings_token().get_token_id(),
            unbond_token_id: self.unbond_token().get_token_id(),
        }
    }

    #[storage_mapper("liquidityReserve")]
    fn liquidity_reserve(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("liquidityNeededForEpoch")]
    fn liquidity_needed_for_epoch(&self, epoch: u64) -> SingleValueMapper<BigUint>;

    #[view(getMinLiquidityReserveNeeded)]
    #[storage_mapper("minLiquidityReserveNeeded")]
    fn min_liquidity_reserve_needed(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("lastUpdateForMinLiqReserveNeeded")]
    fn last_update_for_min_liq_reserve_needed(&self) -> SingleValueMapper<u64>;
}
