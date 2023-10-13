// Code generated by the multiversx-sc multi-contract system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                           31
// Async Callback (empty):               1
// Total number of exported functions:  33

#![no_std]

// Configuration that works with rustc < 1.73.0.
// TODO: Recommended rustc version: 1.73.0 or newer.
#![feature(lang_items)]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    controller
    (
        init => init
        deposit => deposit
        withdraw => withdraw
        unbond => unbond
        claimRewards => claim_rewards
        claimControllerRewards => claim_controller_rewards
        rebalance => rebalance
        addPlatform => add_platform
        setPlatformDistribution => set_platforms_distribution
        setControllerState => set_controller_state
        setFeesDistribution => set_fees_distribution
        setRewardsPerShare => set_reward_per_share
        setMinUnbondEpochs => set_min_unbond_epochs
        getSavingsTokenSupply => savings_token_supply
        getForceUnbondFeesPercentage => force_unbond_fees_percentage
        registerSavingsToken => register_savings_token
        registerUnbondToken => register_unbond_token
        calculateRewardsForGivenPosition => calculate_rewards
        setRewardsPerSharePerBlock => set_rewards_per_share_per_block
        setProduceRewardsEnabled => set_produce_rewards_enabled
        isProduceRewardsEnabled => produce_rewards_enabled
        getLastUpdateBlockNonce => last_update_block_nonce
        getRewardsPerShare => rewards_per_share
        getRewardsPerSharePerBlock => rewards_per_share_per_block
        getPhase => get_phase
        setPhase => set_phase
        setDepositFees => set_deposit_fees
        setWithdrawFees => set_withdraw_fees
        getDepositFees => deposit_fees_percentage
        getWithdrawFees => withdraw_fees_percentage
        setVaultAddress => set_vault_address
        getVaultAddress => vault_addr
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
