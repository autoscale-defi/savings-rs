multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::models::SavingsTokenAttributes;
use multiversx_sc_modules::default_issue_callbacks;

const REWARDS_PRECISION: u64 = 1000000000000; // 1e12

#[multiversx_sc::module]
pub trait RewardsModule: default_issue_callbacks::DefaultIssueCallbacksModule {
    /// Updates the rewards per share based on the current block number and the last time the rewards were updated.
    ///
    /// We use a static rewards per share per block and not a dynamic one based on the supply of the savings tokens (shares).
    /// Then, we don't have to update the rewards per share every time a user deposits or withdraws because the change in the supply
    /// doesn't affect the linear increase of the rewards per share.
    /// The linear increase of the rewards per share is only affected by a manual change of the rewards per share per block.
    ///
    /// This function has to be called:
    /// - right BEFORE we change the rewards per share per block manually
    /// - when we want to compute the rewards for a user (it's not even mandatory but it's a calculation that we would have done
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

    #[view(calculateRewardsForGivenPosition)]
    fn calculate_rewards_view(
        &self,
        savings_token_amount: &BigUint,
        attributes: &SavingsTokenAttributes<Self::Api>,
    ) -> BigUint {
        self.get_real_usdc_rewards_amount(&self.calculate_rewards(savings_token_amount, attributes))
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

    fn get_real_usdc_rewards_amount(&self, big_precision_amount: &BigUint) -> BigUint {
        big_precision_amount / REWARDS_PRECISION
    }

    /// The amount of rewards per share that are produced in one block.
    /// Can only be changed manually by the owner.
    /// This is a variation of the APR.
    #[view(getRewardsPerSharePerBlock)]
    #[storage_mapper("rewardsPerSharePerBlock")]
    fn rewards_per_share_per_block(&self) -> SingleValueMapper<BigUint>;

    #[view(isProduceRewardsEnabled)]
    #[storage_mapper("produceRewardsEnabled")]
    fn produce_rewards_enabled(&self) -> SingleValueMapper<bool>;

    #[view(getLastUpdateBlockNonce)]
    #[storage_mapper("lastUpdateBlockNonce")]
    fn last_update_block_nonce(&self) -> SingleValueMapper<u64>;

    #[view(getRewardsPerShare)]
    #[storage_mapper("rewardsPerShare")]
    fn rewards_per_share(&self) -> SingleValueMapper<BigUint>;
}
