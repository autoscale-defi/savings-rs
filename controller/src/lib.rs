#![no_std]

use token::SavingsTokenAttributes;

multiversx_sc::imports!();

pub mod token;

#[multiversx_sc::contract]
pub trait ControllerContract: token::TokenModule {
    #[init]
    fn init(&self, usdc_token_id: TokenIdentifier) {
        self.usdc_token().set_if_empty(usdc_token_id);
    }

    // ROBIN
    #[payable("*")]
    #[endpoint]
    fn deposit(&self) -> EsdtTokenPayment<Self::Api> {
        let payments = self.call_value().all_esdt_transfers();
        let usdc_payment = payments
            .try_get(0)
            .unwrap_or_else(|| sc_panic!("empty payments"));
        let additional_payments = payments.slice(1, payments.len()).unwrap_or_default();

        self.usdc_token()
            .require_same_token(&usdc_payment.token_identifier);
        self.savings_token()
            .require_all_same_token(&additional_payments);

        let last_bloc = self.blockchain().get_block_nonce();
        let attributes = SavingsTokenAttributes {
            reward_per_share: self.reward_per_share().get(),
            accumulated_rewards: BigUint::zero(),
            last_bloc,
        };

        let caller = self.blockchain().get_caller();
        let new_savings_token = self.create_savings_token_by_merging(
            usdc_payment.amount,
            &attributes,
            &additional_payments,
        );

        self.send()
            .direct_non_zero_esdt_payment(&caller, &new_savings_token);

        new_savings_token
    }

    fn create_savings_token_by_merging(
        &self,
        amount: BigUint,
        _attributes: &SavingsTokenAttributes<Self::Api>,
        payments: &ManagedVec<EsdtTokenPayment<Self::Api>>,
    ) -> EsdtTokenPayment<Self::Api> {
        // merge les attributs
        let merged_attributes = SavingsTokenAttributes {
            reward_per_share: BigUint::zero(), // todo
            accumulated_rewards: BigUint::zero(), //todo
            last_bloc: 0,                      // todo
        };

        // additionner la nouvelle position + les anciennes 
        // burn les anciennes positions 
        // baisser la supply total des savings token 
        // est-ce que je mettrai pas un IF pour la loop et je rentre dedans que si j'ai besoin de merge ? si la len des paiements est de 0 je rentre pas
        let mut new_amount = amount;
        for payment in payments.iter() {
            new_amount += payment.amount.clone();

            self.savings_token().nft_burn(payment.token_nonce, &payment.amount);
            self.savings_token_supply().update(|x| *x -= payment.amount);
        }

        // creer le nouveau savings token
        let new_savings_token = self
            .savings_token()
            .nft_create(new_amount.clone(), &merged_attributes);

        self.savings_token_supply().update(|x| *x += new_amount);
        // je dois merge seulement si il y a des payments - pas de merge sinon

        new_savings_token
    }

    // ROBIN
    #[payable("*")]
    #[endpoint]
    fn withdraw(&self) {}

    // ROBIN
    #[payable("*")]
    #[endpoint(claimRewards)]
    fn claim_rewards(&self) {}

    // NICOLAS
    #[endpoint(claimControllerRewards)]
    fn claim_controller_rewards(&self) {}

    // NICOLAS
    #[endpoint]
    fn rebalance(&self) {}

    // ROBIN
    #[endpoint]
    fn unbond(&self) {}

    // NICOLAS
    #[only_owner]
    #[endpoint(addPlatform)]
    fn add_platform(&self) {}

    // NICOLAS
    #[only_owner]
    #[endpoint(setPlatformDistribution)]
    fn set_platforms_distribution(&self) {
        // quand on change la répartition alors on va withdraw + redeposit all dans cette fonction
    }

    // ROBIN
    fn merge_position(&self) {}

    // NICOLAS
    #[only_owner]
    #[endpoint(setControllerState)]
    fn set_controller_state(&self) {}

    // NICOLAS
    #[only_owner]
    #[endpoint(setFeesDistribution)]
    fn set_fees_distribution(&self) {}

    // DUOQ
    #[only_owner]
    #[endpoint(setRewardsPerShare)]
    fn set_reward_per_share(&self) {

        // est-ce qu'on a besoin du savings_token_supply pour le calculer ? 
    }

    #[view(getSavingsTokenSupply)]
    #[storage_mapper("savingsTokenSupply")]
    fn savings_token_supply(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("rewardPerShare")]
    fn reward_per_share(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("usdcTokenId")]
    fn usdc_token(&self) -> FungibleTokenMapper<Self::Api>;
}
