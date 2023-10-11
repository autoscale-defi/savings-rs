#![no_std]

use token::UnbondTokenAttributes;

multiversx_sc::imports!();

pub mod rewards;
pub mod token;

#[multiversx_sc::contract]
pub trait ControllerContract: token::TokenModule + rewards::RewardsModule {
    #[init]
    fn init(&self, usdc_token_id: TokenIdentifier) {
        self.usdc_token().set_if_empty(usdc_token_id);
    }

    #[payable("*")]
    #[endpoint]
    fn deposit(&self) -> EsdtTokenPayment<Self::Api> {
        let payments = self.call_value().all_esdt_transfers();

        // add fees
        let usdc_payment = payments
            .try_get(0)
            .unwrap_or_else(|| sc_panic!("empty payments"));
        let additional_payments = payments.slice(1, payments.len()).unwrap_or_default();

        self.usdc_token()
            .require_same_token(&usdc_payment.token_identifier);
        self.savings_token()
            .require_all_same_token(&additional_payments);

        let new_savings_token =
            self.create_savings_token_by_merging(usdc_payment.amount.clone(), &additional_payments);

        self.liquidity_reserve()
            .update(|x| *x += usdc_payment.amount);

        let caller = self.blockchain().get_caller();
        self.send()
            .direct_non_zero_esdt_payment(&caller, &new_savings_token);

        new_savings_token
    }

    fn create_savings_token_by_merging(
        &self,
        amount: BigUint,
        payments: &ManagedVec<EsdtTokenPayment<Self::Api>>,
    ) -> EsdtTokenPayment<Self::Api> {
        let mut merged_attributes = self.merge_positions(payments);
        merged_attributes.total_shares += amount.clone();

        self.burn_savings_tokens(&payments);

        let new_savings_token = self
            .savings_token()
            .nft_create(merged_attributes.total_shares.clone(), &merged_attributes);

        new_savings_token
    }

    #[payable("*")]
    #[endpoint]
    fn withdraw(&self) -> ManagedVec<EsdtTokenPayment> {
        let payment = self.call_value().single_esdt();
        self.savings_token()
            .require_same_token(&payment.token_identifier);
        require!(payment.amount > 0, "Payment amount cannot be zero");

        // Do we accept multiple payments or not?
        // create a managedvec for now
        let mut positions = ManagedVec::new();
        positions.push(payment.clone());
        let rewards = self.merge_positions(&positions);

        let current_epoch = self.blockchain().get_block_epoch();
        let min_unbond_epochs = self.min_unbond_epochs().get();

        let unbond_token_attr = UnbondTokenAttributes {
            unlock_epoch: current_epoch + min_unbond_epochs,
        };
        let unbond_token_payment = self
            .unbond_token()
            .nft_create(payment.amount.clone(), &unbond_token_attr);

        self.burn_savings_tokens(&positions);

        let mut output_payments = ManagedVec::new();

        let rewards_payment = EsdtTokenPayment::new(
            self.usdc_token().get_token_id(),
            0,
            rewards.accumulated_rewards,
        );
        output_payments.push(rewards_payment);
        output_payments.push(unbond_token_payment);

        let caller = self.blockchain().get_caller();
        self.send().direct_multi(&caller, &output_payments);

        output_payments
    }

    // ROBIN
    #[endpoint]
    fn unbond(&self) {
        // vérifier que liquidity reserve >= montant a envoyer
        // burn le token d'unbond
        // envoyer les fonds à l'user
        // decrease la liquidity reserve
    }

    // ROBIN
    #[payable("*")]
    #[endpoint(claimRewards)]
    fn claim_rewards(&self) {}

    // NICOLAS
    #[endpoint(claimControllerRewards)]
    fn claim_controller_rewards(&self) {}

    // NICOLAS
    #[endpoint]
    fn rebalance(&self) {}

    // NICOLAS
    #[only_owner]
    #[endpoint(addPlatform)]
    fn add_platform(&self) {}

    // NICOLAS
    #[only_owner]
    #[endpoint(setPlatformDistribution)]
    fn set_platforms_distribution(&self) {
        // quand on change la répartition alors on va withdraw + redeposit all dans cette fonction
    }

    // ROBIN
    fn merge_position(&self) {}

    // NICOLAS
    #[only_owner]
    #[endpoint(setControllerState)]
    fn set_controller_state(&self) {}

    // NICOLAS
    #[only_owner]
    #[endpoint(setFeesDistribution)]
    fn set_fees_distribution(&self) {}

    // DUOQ
    #[only_owner]
    #[endpoint(setRewardsPerShare)]
    fn set_reward_per_share(&self) {

        // est-ce qucalculer ? 'on a besoin du savings_token_supply pour le
    }

    #[endpoint(setMinUnbondEpochs)]
    fn set_min_unbond_epochs(&self, min_unbond_epochs: u64) {
        self.min_unbond_epochs().set(&min_unbond_epochs);
    }

    #[storage_mapper("liquidityReserve")]
    fn liquidity_reserve(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("minUnbondEpochs")]
    fn min_unbond_epochs(&self) -> SingleValueMapper<u64>;

    #[view(getSavingsTokenSupply)]
    #[storage_mapper("savingsTokenSupply")]
    fn savings_token_supply(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("rewardPerShare")]
    fn reward_per_share(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("usdcTokenId")]
    fn usdc_token(&self) -> FungibleTokenMapper<Self::Api>;
}
