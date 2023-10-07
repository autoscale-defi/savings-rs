use crate::storage;
multiversx_sc::imports!();

pub struct EnterFarmResult<M: ManagedTypeApi> {
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
        let mut opt_share_token_payment: Option<EsdtTokenPayment<Self::Api>> = None;
        let mut other_payments = ManagedVec::new();
        for payment in result.iter() {
            if payment.token_identifier == share_token_identifier {
                require!(
                    opt_share_token_payment.is_none(),
                    "Received multiple share tokens"
                );

                opt_share_token_payment = Some(payment)
            } else {
                other_payments.push(payment)
            }
        }

        let Some(share_token_payment) = opt_share_token_payment else {
            sc_panic!("No share payment received");
        };

        EnterFarmResult {
            share_token_payment,
            other_payments,
        }
    }

    fn claim_farm_rewards(
        &self,
        current_position_payment: EsdtTokenPayment<Self::Api>,
        farm_address: &ManagedAddress<Self::Api>
    ) -> ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>> {
        self.holder_proxy(self.holder_address().get())
            .claim_farm_rewards_forward(farm_address)
            .with_esdt_transfer(current_position_payment)
            .execute_on_dest_context()
    }

    #[proxy]
    fn holder_proxy(&self, holder_address: ManagedAddress<Self::Api>) -> proxy::Proxy<Self::Api>;
}