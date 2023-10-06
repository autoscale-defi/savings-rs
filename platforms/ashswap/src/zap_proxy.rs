use crate::storage;
multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TopEncode, NestedEncode, TopDecode, NestedDecode, TypeAbi, PartialEq, Clone)]
pub enum ZapExchange {
    XExchange,
    AshSwap
}

#[derive(TopEncode, TopDecode, TypeAbi, PartialEq, Clone)]
pub struct ZapExchangeInfos<M: ManagedTypeApi> {
    pub exchange: ZapExchange,
    pub override_entry_token: Option<TokenIdentifier<M>>
}

#[derive(TopEncode, TopDecode, TypeAbi, PartialEq, Clone)]
pub struct ZapInResultInfos<M: ManagedTypeApi> {
    pub lp_payment: EsdtTokenPayment<M>,
    pub left_payments: ManagedVec<M, EsdtTokenPayment<M>>
}

mod proxy {
    use crate::zap_proxy::{ZapExchangeInfos, ZapInResultInfos};
    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
    pub trait ZapProxy {
        #[endpoint(zapIn)]
        #[payable("*")]
        fn zap_in(
            &self,
            exchange: &ZapExchangeInfos<Self::Api>,
            pool_address: &ManagedAddress<Self::Api>,
            opt_min_amount: &Option<BigUint<Self::Api>>,
        ) -> ZapInResultInfos<Self::Api>;
    }
}

#[multiversx_sc::module]
pub trait ZapProxyModule: ContractBase
    + storage::StorageModule
{
    fn zap_in_payment(
        &self,
        pool_address: &ManagedAddress<Self::Api>,
        in_payment: EsdtTokenPayment<Self::Api>
    ) -> EsdtTokenPayment<Self::Api> {
        let zap_exchange = self.get_zap_exchange_for_in_token_or_default(
            &in_payment.token_identifier
        );

        let result: ZapInResultInfos<Self::Api> = self.zap_proxy(self.zap_address().get())
            .zap_in(
                &zap_exchange,
                pool_address,
                &None
            )
            .with_esdt_transfer(
                in_payment
            )
            .execute_on_dest_context();

        // Todo : handle left payments
        result.lp_payment
    }

    #[proxy]
    fn zap_proxy(&self, zap_address: ManagedAddress<Self::Api>) -> proxy::Proxy<Self::Api>;
}