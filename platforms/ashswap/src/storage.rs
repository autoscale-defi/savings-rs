use crate::zap_proxy::{ZapExchange, ZapExchangeInfos};
multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TopEncode, NestedEncode, TopDecode, NestedDecode, TypeAbi, ManagedVecItem, Debug)]
pub struct PoolInfos<M: ManagedTypeApi> {
    pub pool_address: ManagedAddress<M>,
    pub farm_address: ManagedAddress<M>,
    pub weight: u64
}

#[multiversx_sc::module]
pub trait StorageModule: ContractBase {

    #[storage_mapper("controller_address")]
    #[view(getControllerAddress)]
    fn controller_address(&self) -> SingleValueMapper<Self::Api, ManagedAddress<Self::Api>>;

    #[storage_mapper("holder_address")]
    #[view(getHolderAddress)]
    fn holder_address(&self) -> SingleValueMapper<Self::Api, ManagedAddress<Self::Api>>;

    #[storage_mapper("asset_token_identifier")]
    #[view(getAssetTokenIdentifier)]
    fn asset_token_identifier(&self) -> SingleValueMapper<Self::Api, TokenIdentifier<Self::Api>>;

    /// This storage is used as a cache to avoid useless contract call to the farm.
    /// Will be populate and clear when pools are added by admins.
    #[storage_mapper("share_token_identifier_for_farm")]
    #[view(getShareTokenIdentifierForFarm)]
    fn share_token_identifier_for_farm(&self, farm_address: &ManagedAddress<Self::Api>) -> SingleValueMapper<Self::Api, TokenIdentifier<Self::Api>>;

    #[storage_mapper("current_position_for_farm")]
    #[view(getCurrentPositionForFarm)]
    fn current_position_for_farm(&self, farm_address: &ManagedAddress<Self::Api>) -> SingleValueMapper<Self::Api, EsdtTokenPayment<Self::Api>>;

    #[storage_mapper("pools")]
    #[view(getPools)]
    fn pools(&self) -> MapMapper<Self::Api, ManagedAddress<Self::Api>, PoolInfos<Self::Api>>;

    #[storage_mapper("pools_total_weight")]
    #[view(getPoolsTotalWeight)]
    fn pools_total_weight(&self) -> SingleValueMapper<Self::Api, u64>;

    #[storage_mapper("zap_address")]
    #[view(getZapAddress)]
    fn zap_address(&self) -> SingleValueMapper<Self::Api, ManagedAddress<Self::Api>>;

    #[storage_mapper("zap_exchange_for_in_token")]
    #[view(getZapExchangeForInToken)]
    fn zap_exchange_for_in_token(&self, token_id: &TokenIdentifier<Self::Api>) -> SingleValueMapper<Self::Api, ZapExchangeInfos<Self::Api>>;

    /// This storage is used as a cache to avoid to handle rewards in the deposit endpoint.
    /// Doing so would lead to a potential out of gas.
    /// In the claim_rewards endpoint, we should append those waiting rewards to the final result.
    #[storage_mapper("waiting_rewards")]
    #[view(getWaitingRewards)]
    fn waiting_rewards(&self) -> VecMapper<Self::Api, EsdtTokenPayment<Self::Api>>;

    fn get_zap_exchange_for_in_token_or_default(
        &self,
        token_id: &TokenIdentifier<Self::Api>
    ) -> ZapExchangeInfos<Self::Api> {
        let mapper = self.zap_exchange_for_in_token(token_id);

        if mapper.is_empty() {
            return ZapExchangeInfos {
                exchange: ZapExchange::AshSwap,
                override_entry_token: None,
            }
        }

        mapper.get()
    }

    fn add_pool(&self, pool: PoolInfos<Self::Api>) {
        self.pools_total_weight().update(|total| *total += pool.weight);

        self.pools()
            .insert(
                pool.pool_address.clone(),
                pool
            );
    }

    fn remove_pool(&self, pool_address: &ManagedAddress<Self::Api>) {
        let Some(removed_pool) = self.pools().remove(pool_address) else {
            sc_panic!("No pool for given address")
        };

        self.pools_total_weight().update(|total| *total -= removed_pool.weight);
    }


}