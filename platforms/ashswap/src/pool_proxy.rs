multiversx_sc::imports!();

mod proxy {
    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait PoolProxy {
        #[view(getLpTokenIdentifier)]
        fn get_lp_token_identifier(&self) -> TokenIdentifier<Self::Api>;
    }
}

#[multiversx_sc::module]
pub trait PoolProxyModule: ContractBase {

    fn get_lp_token_identifier(
        &self,
        pool_address: ManagedAddress<Self::Api>
    ) -> TokenIdentifier<Self::Api> {
        self.pool_proxy(pool_address)
            .get_lp_token_identifier()
            .execute_on_dest_context()
    }

    #[proxy]
    fn pool_proxy(&self, pool_address: ManagedAddress<Self::Api>) -> proxy::Proxy<Self::Api>;

}