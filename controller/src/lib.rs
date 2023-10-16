#![no_std]

use models::{ControllerParametersDTO, PlatformInfo};
use multiversx_sc_modules::default_issue_callbacks;
use phase::Phase;
use token::{SavingsTokenAttributes, UnbondTokenAttributes};

multiversx_sc::imports!();

pub mod models;
pub mod phase;
pub mod rewards;
pub mod token;
pub mod vault_proxy;
pub mod platform_proxy;

const PERCENTAGE_DIVIDER: u64 = 10000;

#[multiversx_sc::contract]
pub trait ControllerContract:
    token::TokenModule
    + rewards::RewardsModule
    + phase::PhaseModule
    + vault_proxy::VaultModule
    + platform_proxy::PlatformModule
    + default_issue_callbacks::DefaultIssueCallbacksModule
{
    // todo add fees_address
    #[init]
    fn init(
        &self,
        usdc_token_id: TokenIdentifier,
        phase: Phase,
        min_unbond_epochs: u64,
        withdraw_fees_perc: u64,
    ) {
        self.usdc_token().set_if_empty(usdc_token_id);
        self.phase().set_if_empty(phase);
        self.min_unbond_epochs().set(min_unbond_epochs);
        self.force_withdraw_fees_percentage()
            .set(&withdraw_fees_perc);
    }

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

    fn charge_and_send_deposit_fees(&self, phase: Phase, amount: &BigUint) -> BigUint {
        let fees_percentage = self.deposit_fees_percentage(phase).get();

        if fees_percentage == 0 {
            return amount.clone();
        }
        let fees_amount = amount * fees_percentage / PERCENTAGE_DIVIDER;
        // send the fees somewhere

        amount - &fees_amount
    }

    fn create_savings_token_by_merging(
        &self,
        amount: &BigUint,
        payments: &ManagedVec<EsdtTokenPayment<Self::Api>>,
    ) -> EsdtTokenPayment<Self::Api> {
        let mut merged_attributes = self.merge_savings_tokens(payments);
        merged_attributes.total_shares += amount.clone();

        self.burn_savings_tokens(payments);

        self.savings_token()
            .nft_create(merged_attributes.total_shares.clone(), &merged_attributes)
    }

    #[payable("*")]
    #[endpoint]
    fn withdraw(&self, opt_force_withdraw: OptionalValue<bool>) -> ManagedVec<EsdtTokenPayment> {
        let payments = self.call_value().all_esdt_transfers();
        self.savings_token().require_all_same_token(&payments);

        let rewards = self.merge_savings_tokens(&payments);
        require!(rewards.total_shares > 0, "Payment amount cannot be zero");

        let current_epoch = self.blockchain().get_block_epoch();
        let min_unbond_epochs = self.min_unbond_epochs().get();

        let force_withdraw = opt_force_withdraw.into_option().unwrap_or(false);

        let mut output_payments = ManagedVec::new();

        if force_withdraw {
            let fees_percentage = self.force_withdraw_fees_percentage().get();
            let fees_amount = rewards.total_shares.clone() * fees_percentage / PERCENTAGE_DIVIDER;
            let savings_token_without_fees = rewards.total_shares.clone() - fees_amount;

            // send the fees somewhere

            output_payments.push(EsdtTokenPayment::new(
                self.usdc_token().get_token_id(),
                0,
                savings_token_without_fees,
            ));
        } else {
            let unbond_token_attr = UnbondTokenAttributes {
                unlock_epoch: current_epoch + min_unbond_epochs,
            };
            let unbond_token_payment = self
                .unbond_token()
                .nft_create(rewards.total_shares.clone(), &unbond_token_attr);

            output_payments.push(unbond_token_payment);

            self.liquidity_needed_for_epoch(unbond_token_attr.unlock_epoch)
                .update(|x| *x += rewards.total_shares.clone());
        }

        self.send_rewards(self.blockchain().get_caller(), rewards.accumulated_rewards);
        self.burn_savings_tokens(&payments);

        let caller = self.blockchain().get_caller();
        self.send().direct_multi(&caller, &output_payments);

        // Rewards are not in the output_payments, maybe we should return it from the vault endpoint first?
        output_payments
    }

    #[payable("*")]
    #[endpoint]
    fn unbond(&self) -> EsdtTokenPayment {
        let payment = self.call_value().single_esdt();
        self.unbond_token()
            .require_same_token(&payment.token_identifier);
        require!(payment.amount > 0, "Payment amount cannot be zero");

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

    #[payable("*")]
    #[endpoint(claimRewards)]
    fn claim_rewards(&self) {
        let payments = self.call_value().all_esdt_transfers();
        self.savings_token().require_all_same_token(&payments);

        let rewards = self.merge_savings_tokens(&payments);
        require!(rewards.total_shares > 0, "Payment amount cannot be zero");

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

        self.send_rewards(caller.clone(), rewards.accumulated_rewards);
        self.send()
            .direct_non_zero_esdt_payment(&caller, &new_savings_token);

        // should return output payments? but same as withdraw, should we first return the payment rewards from the vault?
    }

    #[endpoint(claimControllerRewards)]
    fn claim_controller_rewards(&self) {
        let platforms = self.platforms();

        for platform in platforms.iter() {
            let sc_address = platform.sc_address; // will be used to call the platform sc

            // claimRewards (call platform SC)
            let claim_rewards_payments = self.claim_rewards_for_platform(sc_address);
            let usdc_token_id = self.usdc_token().get_token_id();

            let mut rewards_payment =
                EsdtTokenPayment::new(usdc_token_id.clone(), 0, BigUint::zero());

            for payment in claim_rewards_payments.iter() {
                if payment.token_identifier == usdc_token_id {
                    rewards_payment.amount += payment.amount;
                }
                // send the other payments somewhere ?
            }
            // send rewards to vault  (TAKE A PERCENTAGE FOR US AND THEN SEND TO VAULT)
            self.increase_reserve(rewards_payment);
        }
    }

    // i need to have the minimum liquidity reserve
    // the minimum liquidity reserve is :
    // the liquidity that hasn't been withdraw yet + (on unbond)
    // the liquidity that will be withdraw in the next epoch(s) - need to define how much epochs +
    // a margin liquidity for those who will force withdraw (a fixed margin amount or a percentage of our TVL?)

    // if the total liquid reserve is > than the liquidity we need in the SC
    // we'll invest the difference in the SC platforms following the given plateforms distribution
    // if the reserve liquidity needed id < than the actual liquidity we have in the SC
    // we'll withdraw from the SC platforms following the given plateforms distribution
    #[endpoint(manageLiquidity)]
    fn manage_liquidity(&self) {
        self.update_min_liq_reserve_needed();

        let min_liq_reserve_needed = self.min_liquidity_reserve_needed().get();
        let liquidity_reserve = self.liquidity_reserve().get();
        let liquidity_buffer = self.liquidity_buffer().get();

        let total_liq_reserve = liquidity_reserve + liquidity_buffer;

        if total_liq_reserve > min_liq_reserve_needed {
            let liquidity_diff = total_liq_reserve - min_liq_reserve_needed;
            self.invest(&liquidity_diff);
        } else {
            let liquidity_needed = min_liq_reserve_needed - total_liq_reserve;
            self.withdraw_from_platform_contracts(&liquidity_needed);
        }
    }

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

    // When this function is called, we update the minimum reserved liquidity we need to ensure withdrawals.
    // If the function is not called at every epoch, it loops to update all epochs that have not been updated.
    // As a security, would it be useful to also add the liquidity needed for the current_epoch + 1 to be sure?
    // I think it would be important to do it for at least current_epoch + 2 or 3
    // Otherwise, I think we'll need to withdraw a lot of funds from the investments contracts.
    fn update_min_liq_reserve_needed(&self) {
        let current_epoch = self.blockchain().get_block_epoch();
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
            .set(&current_epoch);
    }

    // We are also supposed to withdraw from all the platforms and then re-deposit with the new distribution.
    // It will be done later.
    #[only_owner]
    #[endpoint(addPlatforms)]
    fn add_platforms(
        &self,
        platforms: MultiValueEncoded<MultiValue3<ManagedBuffer, ManagedAddress, u64>>,
    ) {
        for platform in platforms.into_iter() {
            let (name, sc_address, weight) = platform.into_tuple();

            let platform_info = PlatformInfo {
                name,
                sc_address,
                weight,
            };
            let is_new = self.platforms().insert(platform_info);
            require!(is_new, "Platform already added");

            self.platforms_total_weight().update(|x| *x += weight);
        }
    }

    // I'm not sure this works (Maybe I have to loop on the indexes instead).
    // We are also supposed to withdraw from all the platforms and then re-deposit with the new distribution.
    // It will be done later.
    #[only_owner]
    #[endpoint(removePlatforms)]
    fn remove_platforms(&self, sc_addresses: MultiValueEncoded<ManagedAddress>) {
        for sc_address in sc_addresses.into_iter() {
            let platforms = self.platforms();

            for platform in platforms.iter() {
                if platform.sc_address == sc_address {
                    self.platforms().swap_remove(&platform);
                    self.platforms_total_weight()
                        .update(|x| *x -= platform.weight);
                }
            }
        }
    }

    #[endpoint(setMinUnbondEpochs)]
    fn set_min_unbond_epochs(&self, min_unbond_epochs: u64) {
        self.min_unbond_epochs().set(min_unbond_epochs);
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

    #[storage_mapper("minUnbondEpochs")]
    fn min_unbond_epochs(&self) -> SingleValueMapper<u64>;

    #[view(getPlaforms)]
    #[storage_mapper("platforms")]
    fn platforms(&self) -> UnorderedSetMapper<PlatformInfo<Self::Api>>;

    #[view(getPlatformsTotalWeight)]
    #[storage_mapper("platformsTotalWeight")]
    fn platforms_total_weight(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("liquidityNeededForEpoch")]
    fn liquidity_needed_for_epoch(&self, epoch: u64) -> SingleValueMapper<BigUint>;

    #[storage_mapper("minLiquidityReserveNeeded")]
    fn min_liquidity_reserve_needed(&self) -> SingleValueMapper<BigUint>;

    // In the future, it would be interesting for the liquidity buffer to be dynamic.
    // It would represent a percentage of the total value locked.
    #[storage_mapper("liquidityBuffer")]
    fn liquidity_buffer(&self) -> SingleValueMapper<BigUint>;

    // too long, need a new naming
    #[storage_mapper("lastUpdateForMinLiqReserveNeeded")]
    fn last_update_for_min_liq_reserve_needed(&self) -> SingleValueMapper<u64>;

    #[view(getForceWithdrawFeesPercentage)]
    #[storage_mapper("forceWithdrawFeesPercentage")]
    fn force_withdraw_fees_percentage(&self) -> SingleValueMapper<u64>;
}
