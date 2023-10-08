multiversx_sc::imports!();

mod proxy {
    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait FarmProxy {
        #[view(getFarmTokenId)]
        fn get_farm_token_id(&self) -> TokenIdentifier<Self::Api>;
    }
}

#[multiversx_sc::module]
pub trait FarmProxyModule: ContractBase {

    fn get_farm_token_identifier(
        &self,
        farm_address: ManagedAddress<Self::Api>
    ) -> TokenIdentifier<Self::Api> {
        self.farm_proxy(farm_address)
            .get_farm_token_id()
            .execute_on_dest_context()
    }

    #[proxy]
    fn farm_proxy(&self, farm_address: ManagedAddress<Self::Api>) -> proxy::Proxy<Self::Api>;

}