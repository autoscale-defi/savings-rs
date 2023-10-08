use crate::{farm_proxy, pool_proxy, storage};
use crate::storage::PoolInfos;
use crate::zap_proxy::{ZapExchange, ZapExchangeInfos};
multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait AdminModule: ContractBase
    + storage::StorageModule + pool_proxy::PoolProxyModule + farm_proxy::FarmProxyModule
{
    #[only_owner]
    #[endpoint(addPool)]
    fn add_pool(
        &self,
        pool_address: ManagedAddress<Self::Api>,
        farm_address: ManagedAddress<Self::Api>,
        weight: u64
    ) {
        let lp_token_identifier = self.get_lp_token_identifier(pool_address.clone());
        self.lp_token_identifier_for_pool(&pool_address).set(lp_token_identifier);

        let farm_share_token_identifier = self.get_farm_token_identifier(farm_address.clone());
        self.share_token_identifier_for_farm(&farm_address).set(farm_share_token_identifier);

        let pool = PoolInfos {
            pool_address,
            farm_address,
            weight,
        };

        self.add_pool_in_storage(pool)
    }

    #[only_owner]
    #[endpoint(removePool)]
    fn remove_pool(
        &self
    ) {
        todo!() // after the hackathon
    }

    #[only_owner]
    #[endpoint(setZapStartExchangeForToken)]
    fn set_zap_start_exchange_for_token(
        &self,
        token_identifier: TokenIdentifier<Self::Api>,
        exchange: ZapExchange,
        override_entry_token: Option<TokenIdentifier<Self::Api>>
    ) {
        let zap_exchange = ZapExchangeInfos {
            exchange,
            override_entry_token,
        };

        self.zap_start_exchange_for_token(&token_identifier).set(zap_exchange);
    }
}