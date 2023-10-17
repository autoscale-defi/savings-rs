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

    /// This storage is used as a cache to avoid useless contract call to the pool.
    /// Will be populate and clear when pools are added by admins.
    #[storage_mapper("lp_token_identifier_for_pool")]
    #[view(getShareTokenIdentifierForPool)]
    fn lp_token_identifier_for_pool(&self, farm_address: &ManagedAddress<Self::Api>) -> SingleValueMapper<Self::Api, TokenIdentifier<Self::Api>>;

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

    /// Gives more flexibility to guide the zap to find a valid route for USDC -> token_id.
    /// Default value is provided in the [`get_zap_in_start_exchange_for_lp_token_or_default`] function.
    #[storage_mapper("zap_start_exchange_for_token")]
    #[view(getZapStartExchangeForToken)]
    fn zap_start_exchange_for_token(&self, token_id: &TokenIdentifier<Self::Api>) -> SingleValueMapper<Self::Api, ZapExchangeInfos<Self::Api>>;

    /// This storage is used as a cache to avoid to handle rewards in the deposit endpoint.
    /// Doing so would lead to a potential out of gas.
    /// In the claim_rewards endpoint, we should append those waiting rewards to the final result.
    #[storage_mapper("waiting_rewards")]
    #[view(getWaitingRewards)]
    fn waiting_rewards(&self) -> VecMapper<Self::Api, EsdtTokenPayment<Self::Api>>;

    /// Stores tokens that are swappable to the asset token.
    /// This storage is useful to avoid sending non-swappable tokens to the zap.
    /// Doing so would make the transaction to fail.
    #[storage_mapper("swappable_tokens")]
    fn swappable_tokens(&self) -> WhitelistMapper<Self::Api, TokenIdentifier<Self::Api>>;

    /// TEMP: This storage mapper is only useful to do a quick PoC for the hackathon.
    /// It doesn't represent an exact value of assets held by this contracts.
    /// The actual asset amount can vary for a lot of reasons: slippage, fees, hacks of third-party contracts, etc...
    /// After the hackathon, it'll be replaced by a real computation, using AshSwap/Autoscale contracts' views.
    #[storage_mapper("deposited_assets")]
    fn deposited_assets(&self) -> SingleValueMapper<Self::Api, BigUint<Self::Api>>;

    fn get_zap_start_exchange_for_token_or_default(
        &self,
        token_id: &TokenIdentifier<Self::Api>
    ) -> ZapExchangeInfos<Self::Api> {
        let mapper = self.zap_start_exchange_for_token(token_id);

        if mapper.is_empty() {
            return ZapExchangeInfos {
                exchange: ZapExchange::AshSwap,
                override_entry_token: None,
            }
        }

        mapper.get()
    }

    fn add_pool_in_storage(&self, pool: PoolInfos<Self::Api>) {
        self.pools_total_weight().update(|total| *total += pool.weight);

        self.pools()
            .insert(
                pool.pool_address.clone(),
                pool
            );
    }

    fn remove_pool_from_storage(&self, pool_address: &ManagedAddress<Self::Api>) {
        let Some(removed_pool) = self.pools().remove(pool_address) else {
            sc_panic!("No pool for given address")
        };

        self.pools_total_weight().update(|total| *total -= removed_pool.weight);
    }


}