# The Controller Smart Contract

## Smart Contract Phase

The Controller smart contract can be under 2 different phases:

- _Accumulation_: Savings+ generates more rewards than it distributes to users. This phase accumulates rewards in the Vault smart contract.
- _Depletion_: Savings+ distributes more rewards than it generates. This phase depletes rewards from the Vault smart contract.

## How to interact with the contract as a user

- `deposit` endpoint. Call this endpoint with USDC as payment. You will receive a receipt token as MetaESDT called `Savings` token. This token is a representation of your deposit. There is different deposit fees depending on the phase of the contract. There is possibly way bigger fees during the depletion phase to discourage users to deposit during this phase and let users who deposited during the accumulation phase to get rewards.
- `claimRewards` endpoint. Call this endpoint with Savings token as payment. You will receive the pending rewards related to your deposit and get back new Savings token with updated rewards.
- `withdraw` endpoint. Call this endpoint with Savings token as payment. You will receive the pending rewards related to your deposit. You can ask for a force withdraw if you want to get your USDC back instantly by paying a fee. If you don't want to pay a fee, you can wait for the 7 epochs awaiting period to get your USDC back, in this case you will receive `Unbond` tokens as a representation of your withdrawal.
- `unbond` endpoint. Call this endpoint with Unbond token as payment. If you call it after the 7 epochs awaiting period, you will receive your USDC back.
