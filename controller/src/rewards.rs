multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use super::token;
use crate::token::SavingsTokenAttributes;
use multiversx_sc_modules::default_issue_callbacks;

#[multiversx_sc::module]
pub trait RewardsModule: token::TokenModule + default_issue_callbacks::DefaultIssueCallbacksModule {
    /// Updates the rewards per share based on the current block number and the last time the rewards were updated.
    ///
    /// We use a static rewards per share per block and not a dynamic one based on the supply of the savings tokens (shares).
    /// Then, we don't have to update the rewards per share every time a user deposits or withdraws because the change in the supply
    /// doesn't affect the linear increase of the rewards per share.
    /// The linear increase of the rewards per share is only affected by a manual change of the rewards per share per block.
    ///
    /// This function has to be called when:
    /// - right BEFORE we change the rewards per share per block manually
    /// - we want to compute the rewards for a user (it's not even mandatory but it's a calculation that we would have done
    ///   on the fly anyway. We will use the compute rewards function also as the view function to get the rewards of a user in real time off-chain)
    fn update_rewards_per_share(&self) {
        let last_update_block_nonce = self.last_update_block_nonce().get();
        let current_block_nonce = self.blockchain().get_block_nonce();
        
        let rewards_enabled = self.produce_rewards_enabled().get();
        let blocks_since_last_update = current_block_nonce - last_update_block_nonce;

        if blocks_since_last_update > 0 && rewards_enabled {
            let computed_rewards_per_share_since_last_update = self
                .rewards_per_share_per_block()
                .get()
                .mul(blocks_since_last_update);

            self.rewards_per_share().update(|x| {
                *x += computed_rewards_per_share_since_last_update;
            });
        }

        self.last_update_block_nonce().set(current_block_nonce);
    }

    /// Calculate the rewards for a given position.
    /// It sums up the rewards and the accumulated rewards based on the proportion of the shares given.
    ///
    /// This function has to be called when:
    /// - you want to know off-chain the rewards of a given position
    /// - A user claims his rewards
    /// - We merge multiple positions into one (the sum of the old positions ends up in the accumulated rewards of the new position)
    #[view(calculateRewardsForGivenPosition)]
    fn calculate_rewards(
        &self,
        savings_token_amount: &BigUint,
        attributes: &SavingsTokenAttributes<Self::Api>,
    ) -> BigUint {
        self.update_rewards_per_share();

        let mut rewards = savings_token_amount
            * &(self.rewards_per_share().get() - &attributes.initial_rewards_per_share);

        rewards +=
            &attributes.accumulated_rewards * savings_token_amount / &attributes.total_shares;

        rewards
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

    #[only_owner]
    #[endpoint(setRewardsPerSharePerBlock)]
    fn set_rewards_per_share_per_block(&self, new_rewards_per_share_per_block: BigUint) {
        self.update_rewards_per_share();
        self.rewards_per_share_per_block()
            .set(new_rewards_per_share_per_block);
    }

    #[only_owner]
    #[endpoint(setProduceRewardsEnabled)]
    fn set_produce_rewards_enabled(&self, enabled: bool) {
        self.update_rewards_per_share();
        self.produce_rewards_enabled().set(enabled);
    }

    #[view(isProduceRewardsEnabled)]
    #[storage_mapper("produceRewardsEnabled")]
    fn produce_rewards_enabled(&self) -> SingleValueMapper<bool>;

    #[view(getLastUpdateBlockNonce)]
    #[storage_mapper("lastUpdateBlockNonce")]
    fn last_update_block_nonce(&self) -> SingleValueMapper<u64>;

    #[view(getRewardsPerShare)]
    #[storage_mapper("rewardsPerShare")]
    fn rewards_per_share(&self) -> SingleValueMapper<BigUint>;

    /// The amount of rewards per share that are produced in one block.
    /// Can only be changed manually by the owner.
    /// This is a variation of the APR.
    #[view(getRewardsPerSharePerBlock)]
    #[storage_mapper("rewardsPerSharePerBlock")]
    fn rewards_per_share_per_block(&self) -> SingleValueMapper<BigUint>;
}
