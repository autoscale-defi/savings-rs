# Savings+ by Autoscale Smart Contracts

Welcome to the Savings+ smart contracts repository by Autoscale. You can find here the [technical specifications](https://docs.google.com/document/d/1c1ayR6Yxo5nqzNyRjX9pfLqY6PH3B73JMhVxSzIN72o/edit?usp=sharing) and the [documentation](https://knowing-dewberry-166.notion.site/Savings-by-Autoscale-cd3f225c831d4fc1bff8bb590cad9824?pvs=4).

This repository comprises three distinct types of smart contracts:

1. [**Controller Contract**](controller/README.md) - Orchestrates the interaction between users and other contracts.
2. [**Vault Contract**](vault/README.md) - Responsible for holding all rewards.
3. [**Platforms Contract**](platforms/README.md) - Formalizes the interaction between the controller and the various DeFi platforms.

## Frontend Repository

For the frontend of Savings+, please visit [the savings-interface repository](https://github.com/autoscale-defi/savings-interface).

## Interactions

Interactions and deployment flow for the controller and vault contracts can be found in the [interaction](interaction) directory.

Interactions for the platforms contracts can be found in the `interaction` folder in the specific platform directory (e.g. [AshSwap](platforms/ashswap/interaction))

## APR Calculator

By running the command `python3 scripts/aprCalculator.py`, you can do some conversions and also calculate the amount of rewards for a given duration and APR.
