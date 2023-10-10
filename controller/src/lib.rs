#![no_std]

multiversx_sc::imports!();


#[multiversx_sc::contract]
pub trait ControllerContract {
    #[init]
    fn init(&self) {}

    // ROBIN
    #[payable("*")]
    #[endpoint]
    fn deposit(&self) {}

    // ROBIN
    #[payable("*")]
    #[endpoint]
    fn withdraw(&self) {}

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

    // ROBIN
    #[endpoint]
    fn unbond(&self) {}

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
    #[only_owner]
    #[payable("EGLD")]
    #[endpoint(issueAndSetLocalRoles)]
    fn issue_and_set_local_roles(&self) {}

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
    fn set_rewards_per_share(&self) {}
}
