multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait VaultModule {
    fn send_rewards(&self, destination: ManagedAddress, amount: BigUint) {
        self.vault_proxy(self.vault_addr().get())
            .send_rewards(destination, amount)
            .execute_on_dest_context::<IgnoreValue>();
    }

    #[proxy]
    fn vault_proxy(&self, sc_address: ManagedAddress) -> vault::Proxy<Self::Api>;

    #[only_owner]
    #[endpoint(setVaultAddress)]
    fn set_vault_address(&self, vault_addr: ManagedAddress) {
        self.vault_addr().set(&vault_addr);
    }

    #[view(getVaultAddress)]
    #[storage_mapper("vaultAddr")]
    fn vault_addr(&self) -> SingleValueMapper<ManagedAddress>;
}
