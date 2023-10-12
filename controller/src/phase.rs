multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode)]
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

    #[storage_mapper("phase")]
    fn phase(&self) -> SingleValueMapper<Phase>;

    #[view(getDepositFeesPercentageOnDepletion)]
    #[storage_mapper("depositFeesPercentageOnDepletion")]
    fn deposit_fees_percentage_on_depletion(&self) -> SingleValueMapper<u64>;
}
