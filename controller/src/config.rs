multiversx_sc::imports!();

use crate::models::{Phase, PlatformInfo};

#[multiversx_sc::module]
pub trait ConfigModule {
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
                    break;
                }
            }
        }
    }

    #[only_owner]
    #[endpoint(setLiquidityBuffer)]
    fn set_liquidity_buffer(&self, liq_buffer_amount: BigUint) {
        self.liquidity_buffer().set(&liq_buffer_amount);
    }

    #[only_owner]
    #[endpoint(setForceWithdrawFeesPercentage)]
    fn set_force_withdraw_fees_percentage(&self, withdraw_fees_perc: u64) {
        self.force_withdraw_fees_percentage()
            .set(withdraw_fees_perc);
    }

    #[only_owner]
    #[endpoint(setVaultAddress)]
    fn set_vault_address(&self, vault_addr: ManagedAddress) {
        self.vault_addr().set(&vault_addr);
    }

    #[endpoint(setMinUnbondEpochs)]
    fn set_min_unbond_epochs(&self, min_unbond_epochs: u64) {
        self.min_unbond_epochs().set(min_unbond_epochs);
    }

    #[only_owner]
    #[endpoint(setFeesAddress)]
    fn set_fees_address(&self, fees_address: ManagedAddress) {
        self.fees_address().set(&fees_address);
    }

    #[only_owner]
    #[endpoint(setPhase)]
    fn set_phase(&self, phase: Phase) {
        require!(
            !self.deposit_fees_percentage(phase.clone()).is_empty(),
            "Need to set the deposit fees before changing phase"
        );
        self.phase().set(phase);
    }

    #[only_owner]
    #[endpoint(setDepositFees)]
    fn set_deposit_fees(&self, phase: Phase, fees_perc: u64) {
        self.deposit_fees_percentage(phase).set(fees_perc);
    }

    #[only_owner]
    #[endpoint(setPerformanceFees)]
    fn set_performance_fees(&self, fees_perc: u64) {
        self.performance_fees().set(fees_perc);
    }

    /// Will maybe be calculated dynamically in the future.
    #[view(getPhase)]
    fn get_phase(&self) -> Phase {
        self.phase().get()
    }

    #[view(getPlaforms)]
    #[storage_mapper("platforms")]
    fn platforms(&self) -> UnorderedSetMapper<PlatformInfo<Self::Api>>;

    #[view(getPlatformsTotalWeight)]
    #[storage_mapper("platformsTotalWeight")]
    fn platforms_total_weight(&self) -> SingleValueMapper<u64>;

    #[view(getForceWithdrawFeesPercentage)]
    #[storage_mapper("forceWithdrawFeesPercentage")]
    fn force_withdraw_fees_percentage(&self) -> SingleValueMapper<u64>; // todo

    /// In the future, it would be interesting for the liquidity buffer to be dynamic.
    /// It would represent a percentage of the total value locked.
    #[storage_mapper("liquidityBuffer")]
    fn liquidity_buffer(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("minUnbondEpochs")]
    fn min_unbond_epochs(&self) -> SingleValueMapper<u64>;

    #[view(getVaultAddress)]
    #[storage_mapper("vaultAddr")]
    fn vault_addr(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("feesAddress")]
    fn fees_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("performanceFees")]
    fn performance_fees(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("usdcTokenId")]
    fn usdc_token(&self) -> FungibleTokenMapper<Self::Api>;

    #[storage_mapper("phase")]
    fn phase(&self) -> SingleValueMapper<Phase>;

    #[view(getDepositFees)]
    #[storage_mapper("depositFeesPercentage")]
    fn deposit_fees_percentage(&self, phase: Phase) -> SingleValueMapper<u64>;
}
