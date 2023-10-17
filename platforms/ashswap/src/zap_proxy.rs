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

        #[endpoint(zapOut)]
        #[payable("*")]
        fn zap_out(
            &self,
            exchange: &ZapExchangeInfos<Self::Api>,
            pool_address: &ManagedAddress<Self::Api>,
            dest_token: &TokenIdentifier<Self::Api>,
            opt_min_amount: &Option<BigUint<Self::Api>>,
            should_unwrap_egld: bool
        ) -> EgldOrEsdtTokenPayment<Self::Api>;

        #[endpoint(swap)]
        #[payable("*")]
        fn swap(
            &self,
            start_exchange: &ZapExchangeInfos<Self::Api>,
            dest_token: &TokenIdentifier<Self::Api>,
            opt_min_amount: &Option<BigUint<Self::Api>>,
            should_unwrap_egld: bool
        ) -> EgldOrEsdtTokenPayment<Self::Api>;
    }
}

#[multiversx_sc::module]
pub trait ZapProxyModule: ContractBase
    + storage::StorageModule
{
    fn swap_payment(
        &self,
        in_payment: EsdtTokenPayment<Self::Api>,
        out_token: &TokenIdentifier<Self::Api>
    ) -> EsdtTokenPayment<Self::Api> {
        let start_exchange = self.get_zap_start_exchange_for_token_or_default(
            &in_payment.token_identifier
        );

        self.zap_proxy(self.zap_address().get())
            .swap(
                start_exchange,
                out_token,
                &None,
                false
            )
            .with_esdt_transfer(in_payment)
            .execute_on_dest_context::<EgldOrEsdtTokenPayment<Self::Api>>()
            .unwrap_esdt()
    }

    fn zap_in_payment(
        &self,
        pool_address: &ManagedAddress<Self::Api>,
        in_payment: EsdtTokenPayment<Self::Api>
    ) -> EsdtTokenPayment<Self::Api> {
        let start_exchange = self.get_zap_start_exchange_for_token_or_default(
            &self.lp_token_identifier_for_pool(pool_address).get()
        );

        let result: ZapInResultInfos<Self::Api> = self.zap_proxy(self.zap_address().get())
            .zap_in(
                &start_exchange,
                pool_address,
                &None
            )
            .with_esdt_transfer(
                in_payment
            )
            .execute_on_dest_context();

        // todo after hackathon : handle left payments
        result.lp_payment
    }

    fn zap_out_payment(
        &self,
        pool_address: &ManagedAddress<Self::Api>,
        dest_token: &TokenIdentifier<Self::Api>,
        in_payment: EsdtTokenPayment<Self::Api>
    ) -> EsdtTokenPayment<Self::Api> {
        let start_exchange = self.get_zap_start_exchange_for_token_or_default(
            &self.lp_token_identifier_for_pool(pool_address).get()
        );

        // the result token won't be EGLD, hence no need to use the EgldOrEsdtTokenPayment struct
        let result: EsdtTokenPayment<Self::Api> = self.zap_proxy(self.zap_address().get())
            .zap_out(
                &start_exchange,
                pool_address,
                dest_token,
                &None,
                false
            )
            .with_esdt_transfer(
                in_payment
            )
            .execute_on_dest_context();

        result
    }

    #[proxy]
    fn zap_proxy(&self, zap_address: ManagedAddress<Self::Api>) -> proxy::Proxy<Self::Api>;
}