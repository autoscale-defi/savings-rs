multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait PlatformModule {

    #[proxy]
    fn platform_proxy(&self, sc_address: ManagedAddress) -> platform::Proxy<Self::Api>;
}