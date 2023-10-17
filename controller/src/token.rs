multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use super::rewards;
use crate::models::SavingsTokenAttributes;
use multiversx_sc_modules::default_issue_callbacks;

#[multiversx_sc::module]
pub trait TokenModule:
    rewards::RewardsModule + default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(registerSavingsToken)]
    fn register_savings_token(
        &self,
        token_display_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        num_decimals: usize,
    ) {
        let issue_cost = self.call_value().egld_value();
        self.savings_token().issue_and_set_all_roles(
            EsdtTokenType::Meta,
            issue_cost.clone_value(),
            token_display_name,
            token_ticker,
            num_decimals,
            None,
        );
    }

    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(registerUnbondToken)]
    fn register_unbond_token(
        &self,
        token_display_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        num_decimals: usize,
    ) {
        let issue_cost = self.call_value().egld_value();
        self.unbond_token().issue_and_set_all_roles(
            EsdtTokenType::Meta,
            issue_cost.clone_value(),
            token_display_name,
            token_ticker,
            num_decimals,
            None,
        );
    }

    /// Merges multiple positions into one.
    ///
    /// if called from the deposit function, this function will be called to merge the old positions into one.
    /// You will still have to:
    /// - burn the old positions
    /// - sum up the total_shares of this new position with the amount of the new USDC position
    /// - mint the new position
    ///
    /// if called from the withdraw or claim function, this function will be called to merge all the positions into one.
    /// You will still have to:
    /// - burn the old positions
    /// - take the new position and send it as rewards to the user
    /// - For withdraw:
    ///         - Create the unbond token position
    /// - For claim:
    ///         - Create a new position with the same amount of shares as the old positions
    ///
    /// if called from a "MergePositions" endpoint, this function will be called to merge all the positions into one.
    /// You will still have to:
    /// - burn the old positions
    /// - Mint the new position returned by this function
    /// - Send the new position to the user
    fn merge_savings_tokens(
        &self,
        payments: &ManagedVec<EsdtTokenPayment<Self::Api>>,
    ) -> SavingsTokenAttributes<Self::Api> {
        let mut new_accumulated_rewards = BigUint::zero();
        let mut new_total_shares = BigUint::zero();

        for payment in payments.into_iter() {
            let savings_token_attr: SavingsTokenAttributes<Self::Api> = self
                .savings_token()
                .get_token_attributes(payment.token_nonce);

            new_accumulated_rewards += self.calculate_rewards(&payment.amount, &savings_token_attr);
            new_total_shares += payment.amount;
        }

        SavingsTokenAttributes {
            initial_rewards_per_share: self.rewards_per_share().get(),
            accumulated_rewards: new_accumulated_rewards,
            total_shares: new_total_shares,
        }
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

    fn burn_savings_tokens(&self, tokens: &ManagedVec<EsdtTokenPayment<Self::Api>>) {
        for token in tokens.iter() {
            self.savings_token()
                .nft_burn(token.token_nonce, &token.amount);
        }
    }

    #[view(getSavingsTokenId)]
    #[storage_mapper("savingsTokenId")]
    fn savings_token(&self) -> NonFungibleTokenMapper;

    #[view(getUnbondToken)]
    #[storage_mapper("unbondToken")]
    fn unbond_token(&self) -> NonFungibleTokenMapper;
}
