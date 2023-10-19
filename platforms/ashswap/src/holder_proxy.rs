use crate::storage;
multiversx_sc::imports!();

pub struct EnterFarmResult<M: ManagedTypeApi> {
    pub share_token_payment: EsdtTokenPayment<M>,
    pub other_payments: ManagedVec<M, EsdtTokenPayment<M>>
}

pub struct ExitFarmResult<M: ManagedTypeApi> {
    pub lp_token_payment: EsdtTokenPayment<M>,
    pub other_payments: ManagedVec<M, EsdtTokenPayment<M>>
}

pub struct ClaimRewardsResult<M: ManagedTypeApi> {
    pub share_token_payment: EsdtTokenPayment<M>,
    pub other_payments: ManagedVec<M, EsdtTokenPayment<M>>
}

mod proxy {
    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait AshswapHolderProxy {
        #[payable("*")]
        #[endpoint(enterFarmForward)]
        fn enter_farm_forward(
            &self,
            farm_address: ManagedAddress<Self::Api>
        ) -> ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>>;

        #[payable("*")]
        #[endpoint(exitFarmForward)]
        fn exit_farm_forward(
            &self,
            farm_address: ManagedAddress<Self::Api>
        ) -> ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>>;

        #[payable("*")]
        #[endpoint(claimFarmRewardsForward)]
        fn claim_farm_rewards_forward(
            &self,
            farm_address: ManagedAddress<Self::Api>
        ) -> ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>>;
    }
}

#[multiversx_sc::module]
pub trait HolderProxyModule: ContractBase
    + storage::StorageModule
{
    fn enter_farm(
        &self,
        lp_payment: EsdtTokenPayment<Self::Api>,
        opt_current_position_payment: Option<EsdtTokenPayment<Self::Api>>,
        farm_address: &ManagedAddress<Self::Api>
    ) -> EnterFarmResult<Self::Api> {
        let mut transfer = ManagedVec::from_single_item(lp_payment);
        if let Some(current_position_payment) = opt_current_position_payment {
            transfer.push(current_position_payment);
        }

        let result: ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>> = self.holder_proxy(self.holder_address().get())
            .enter_farm_forward(farm_address)
            .with_multi_token_transfer(transfer)
            .execute_on_dest_context();

        let share_token_identifier = self.share_token_identifier_for_farm(farm_address).get();
        let separated_payments = self.separate_specific_payment_from_another_ones(
            &result,
            &share_token_identifier
        );

        EnterFarmResult {
            share_token_payment: separated_payments.0,
            other_payments: separated_payments.1,
        }
    }

    fn exit_farm_forward(
        &self,
        position_payment: EsdtTokenPayment<Self::Api>,
        pool_lp_token_identifier: &TokenIdentifier<Self::Api>,
        farm_address: &ManagedAddress<Self::Api>
    ) -> ExitFarmResult<Self::Api> {
        let result: ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>> = self.holder_proxy(self.holder_address().get())
            .exit_farm_forward(farm_address)
            .with_esdt_transfer(position_payment)
            .execute_on_dest_context();

        let separated_payments = self.separate_specific_payment_from_another_ones(
            &result,
            pool_lp_token_identifier
        );

        ExitFarmResult {
            lp_token_payment: separated_payments.0,
            other_payments: separated_payments.1,
        }
    }

    fn claim_farm_rewards(
        &self,
        current_position_payment: EsdtTokenPayment<Self::Api>,
        farm_address: &ManagedAddress<Self::Api>
    ) -> ClaimRewardsResult<Self::Api> {
        let result: ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>> = self.holder_proxy(self.holder_address().get())
            .claim_farm_rewards_forward(farm_address)
            .with_esdt_transfer(current_position_payment)
            .execute_on_dest_context();

        let share_token_identifier = self.share_token_identifier_for_farm(farm_address).get();
        let separated_payments = self.separate_specific_payment_from_another_ones(
            &result,
            &share_token_identifier
        );


        ClaimRewardsResult {
            share_token_payment: separated_payments.0,
            other_payments: separated_payments.1,
        }
    }

    /// Separates a specific payment from an array of payments based on the provided token identifier.
    ///
    /// This function scans through the provided payments, identifying and isolating the payment with
    /// the specified token identifier. If the desired payment isn't present or if there are multiple
    /// instances of it, the function reverts the transaction.
    ///
    /// # Parameters
    /// - `payments`: A collection of `EsdtTokenPayment` objects from which the specific payment needs to be extracted.
    /// - `specific_token_identifier`: The unique token identifier of the payment to be separated from the others.
    ///
    /// # Returns
    /// A tuple containing:
    /// 1. The `EsdtTokenPayment` object corresponding to the specified token identifier.
    /// 2. A `ManagedVec` of `EsdtTokenPayment` objects representing all other payments.
    ///
    fn separate_specific_payment_from_another_ones(
        &self,
        payments: &ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>>,
        specific_token_identifier: &TokenIdentifier<Self::Api>
    ) -> (EsdtTokenPayment<Self::Api>, ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>>) {
        let mut opt_specific_token_payment: Option<EsdtTokenPayment<Self::Api>> = None;
        let mut other_payments = ManagedVec::new();
        for payment in payments.iter() {
            if &payment.token_identifier == specific_token_identifier {
                require!(
                    opt_specific_token_payment.is_none(),
                    "Received multiple share tokens"
                );

                opt_specific_token_payment = Some(payment)
            } else {
                other_payments.push(payment)
            }
        }

        let Some(specific_token_payment) = opt_specific_token_payment else {
            sc_panic!("No payment received for token: {}", specific_token_identifier);
        };

        (
            specific_token_payment,
            other_payments
        )
    }

    #[proxy]
    fn holder_proxy(&self, holder_address: ManagedAddress<Self::Api>) -> proxy::Proxy<Self::Api>;
}