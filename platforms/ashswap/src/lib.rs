#![no_std]

mod zap_proxy;
mod storage;
mod holder_proxy;

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait AshSwapPlatformContract: ContractBase
    + storage::StorageModule + zap_proxy::ZapProxyModule + holder_proxy::HolderProxyModule
{
    #[endpoint]
    #[payable("*")]
    fn deposit(&self) {
        let caller = self.blockchain().get_caller();

        require!(
            caller == self.controller_address().get(),
            "Only the controller can call this endpoint"
        );

        let payment = self.call_value().single_esdt();
        let asset_token_identifier = self.asset_token_identifier().get();

        require!(
            payment.token_identifier == asset_token_identifier,
            "Wrong token payment"
        );

        let pools_mapper = self.pools();
        let pools = pools_mapper.values();

        for pool in pools {

        }
    }
}
