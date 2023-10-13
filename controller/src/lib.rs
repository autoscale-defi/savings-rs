#![no_std]

use models::ControllerParametersDTO;
use phase::Phase;
use token::{SavingsTokenAttributes, UnbondTokenAttributes};

multiversx_sc::imports!();

pub mod models;
pub mod phase;
pub mod rewards;
pub mod token;
pub mod vault_proxy;

const PERCENTAGE_DIVIDER: u64 = 10000;

#[multiversx_sc::contract]
pub trait ControllerContract:
    token::TokenModule + rewards::RewardsModule + phase::PhaseModule + vault_proxy::VaultModule
{
    #[init]
    fn init(&self, usdc_token_id: TokenIdentifier, phase: Phase) {
        self.usdc_token().set_if_empty(usdc_token_id);
        self.phase().set_if_empty(phase);
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

    // maybe we can do this for the deposit & the withdraw ? Do we add if its for the deposit or the withdraw in args ?
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
            let fees_percentage = self.force_unbond_fees_percentage().get();
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
        }

        // where do we check if there is enough rewards in the vault?
        // tx will fail anyways but a require! could be nice (in the vault?)
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
        // where do we check if there is enough rewards in the vault?
        // tx will fail anyways but a require! could be nice (in the vault?)
        self.send_rewards(caller.clone(), rewards.accumulated_rewards);
        self.send()
            .direct_non_zero_esdt_payment(&caller, &new_savings_token);

        // should return output payments? but same as withdraw, should we first return the payment rewards from the vault?
    }

    #[endpoint(claimControllerRewards)]
    fn claim_controller_rewards(&self) {}

    #[endpoint]
    fn rebalance(&self) {}

    #[only_owner]
    #[endpoint(addPlatform)]
    fn add_platform(&self) {}

    #[only_owner]
    #[endpoint(setPlatformDistribution)]
    fn set_platforms_distribution(&self) {
        // quand on change la répartition alors on va withdraw + redeposit all dans cette fonction
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
            force_unbond_fees_percentage: self.force_unbond_fees_percentage().get(),
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

    #[storage_mapper("usdcTokenId")]
    fn usdc_token(&self) -> FungibleTokenMapper<Self::Api>;

    #[view(getForceUnbondFeesPercentage)]
    #[storage_mapper("forceUnbondFeesPercentage")]
    fn force_unbond_fees_percentage(&self) -> SingleValueMapper<u64>;
}
