multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode)]
pub enum Phase {
    Accumulation,
    Depletion,
}

#[multiversx_sc::module]
pub trait PhaseModule {
    /// Will maybe be calculated dynamically in the future
    #[view(getPhase)]
    fn get_phase(&self) -> Phase {
        self.phase().get()
    }

    #[only_owner]
    #[endpoint(setPhase)]
    fn set_phase(&self, phase: Phase) {
        self.phase().set(phase);
    }

    // maybe one function setFees with deposit and phase is better?

    // maybe also one mapper ? we can add in the args if its for the deposit or withdraw
    #[only_owner]
    #[endpoint(setDepositFees)]
    fn set_deposit_fees(&self, phase: Phase, fees_perc: u64) {
        self.deposit_fees_percentage(phase).set(fees_perc);
    }

    #[only_owner]
    #[endpoint(setWithdrawFees)]
    fn set_withdraw_fees(&self, phase: Phase, fees_perc: u64) {
        self.withdraw_fees_percentage(phase).set(fees_perc);
    }

    #[storage_mapper("phase")]
    fn phase(&self) -> SingleValueMapper<Phase>;

    #[view(getDepositFees)]
    #[storage_mapper("depositFeesPercentage")]
    fn deposit_fees_percentage(&self, phase: Phase) -> SingleValueMapper<u64>;

    #[view(getWithdrawFees)]
    #[storage_mapper("withdrawFeesPercentage")]
    fn withdraw_fees_percentage(&self, phase: Phase) -> SingleValueMapper<u64>;
}
