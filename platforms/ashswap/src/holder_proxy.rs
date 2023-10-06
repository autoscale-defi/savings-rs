use crate::storage;
multiversx_sc::imports!();

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
    }
}

#[multiversx_sc::module]
pub trait HolderProxyModule: ContractBase
    + storage::StorageModule
{
    fn enter_farm(
        &self,
        lp_payment: EsdtTokenPayment<Self::Api>,
        farm_address: &ManagedAddress<Self::Api>
    ) -> ManagedVec<Self::Api, EsdtTokenPayment<Self::Api>> {
        self.holder_proxy(self.holder_address().get())
            .enter_farm_forward(farm_address)
            .with_esdt_transfer(lp_payment)
            .execute_on_dest_context()
    }

    #[proxy]
    fn holder_proxy(&self, holder_address: ManagedAddress<Self::Api>) -> proxy::Proxy<Self::Api>;
}