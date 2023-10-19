PROXY="https://devnet-gateway.multiversx.com"
CHAIN="D"
OWNER="swallet.pem"

CONTROLLER="controller/output/controller.wasm"
VAULT="vault/output/vault.wasm"

CONTROLLER_ADDR="erd1qqqqqqqqqqqqqpgqzzlkdyqvgkvwmapyupauckfmy6krwh4ef53sf50atn"
VAULT_ADDR="erd1qqqqqqqqqqqqqpgqufehmuwph0247z2t8l2u9h4zndna8ke8f53spznyz5"

USDC_TOKEN_ID="str:USDC-8d4068"
PHASE=0
MIN_UNBOND_EPOCHS=7
DEPOSIT_FEES_CURRENT_PHASE=100
PERFORMANCE_FEES=250
FORCE_WITHDRAW_FEES=750

deployController() {
  mxpy --verbose contract deploy --bytecode="$CONTROLLER" --recall-nonce \
        --pem=$OWNER \
        --gas-limit=599000000 \
        --proxy=$PROXY --chain=$CHAIN \
        --metadata-payable-by-sc \
        --arguments $USDC_TOKEN_ID $PHASE $MIN_UNBOND_EPOCHS $DEPOSIT_FEES_CURRENT_PHASE $PERFORMANCE_FEES $FORCE_WITHDRAW_FEES \
        --outfile="deploy-devnet.interaction.json" --send || return
}

deployVault() {
  mxpy --verbose contract deploy --bytecode="$VAULT" --recall-nonce \
        --pem=$OWNER \
        --gas-limit=599000000 \
        --proxy=$PROXY --chain=$CHAIN \
        --metadata-payable-by-sc \
        --arguments $USDC_TOKEN_ID $CONTROLLER_ADDR \
        --outfile="deploy-devnet.interaction.json" --send || return
}

upgradeController() {
    mxpy --verbose contract upgrade ${CONTROLLER_ADDR} --bytecode="$CONTROLLER" --recall-nonce \
    --pem=${OWNER} \
    --gas-limit=599000000 \
    --proxy=${PROXY} --chain=${CHAIN} \
    --metadata-payable-by-sc \
    --arguments $USDC_TOKEN_ID $PHASE $MIN_UNBOND_EPOCHS $DEPOSIT_FEES_CURRENT_PHASE $PERFORMANCE_FEES $FORCE_WITHDRAW_FEES \
    --send --outfile="deploy-devnet.interaction.json" || return

    echo "Smart contract upgraded address: ${ADDRESS}"
}

upgradeVault() {
    mxpy --verbose contract upgrade ${VAULT_ADDR} --bytecode="$VAULT" --recall-nonce \
    --pem=${OWNER} \
    --gas-limit=599000000 \
    --proxy=${PROXY} --chain=${CHAIN} \
    --metadata-payable-by-sc \
    --arguments $USDC_TOKEN_ID $CONTROLLER_ADDR \
    --send --outfile="deploy-devnet.interaction.json" || return

    echo "Smart contract upgraded address: ${ADDRESS}"
}

registerSavingsToken() {
    NAME="str:AutoscaleSavingsUSDC"
    TICKER="str:ASUSDC"
    DECIMALS=6

    mxpy --verbose contract call ${CONTROLLER_ADDR} --recall-nonce \
          --pem=${OWNER} \
          --proxy=${PROXY} --chain=${CHAIN} \
          --gas-limit=100000000 \
          --value=50000000000000000 \
          --function="registerSavingsToken" \
          --arguments $NAME $TICKER $DECIMALS \
          --send || return
}

registerUnbondToken() {
    NAME="str:AutoscaleSavingsUSDC"
    TICKER="str:ASUUSDC"
    DECIMALS=6

    mxpy --verbose contract call ${CONTROLLER_ADDR} --recall-nonce \
          --pem=${OWNER} \
          --proxy=${PROXY} --chain=${CHAIN} \
          --gas-limit=100000000 \
          --value=50000000000000000 \
          --function="registerUnbondToken" \
          --arguments $NAME $TICKER $DECIMALS \
          --send || return
}

setRewardsPerSharePerBlock() {
    new_rewards_per_share_per_bloc=7610

    mxpy --verbose contract call ${CONTROLLER_ADDR} --recall-nonce \
          --pem=${OWNER} \
          --proxy=${PROXY} --chain=${CHAIN} \
          --gas-limit=100000000 \
          --function="setRewardsPerSharePerBlock" \
          --arguments $new_rewards_per_share_per_bloc \
          --send || return
}

setProduceRewardsEnabled() {
    bool=1

    mxpy --verbose contract call ${CONTROLLER_ADDR} --recall-nonce \
          --pem=${OWNER} \
          --proxy=${PROXY} --chain=${CHAIN} \
          --gas-limit=100000000 \
          --function="setProduceRewardsEnabled" \
          --arguments $bool \
          --send || return
}

setVaultAddress() {
    mxpy --verbose contract call ${CONTROLLER_ADDR} --recall-nonce \
          --pem=${OWNER} \
          --proxy=${PROXY} --chain=${CHAIN} \
          --gas-limit=100000000 \
          --function="setVaultAddress" \
          --arguments $VAULT_ADDR \
          --send || return 
}

addPlatforms() {
    mxpy --verbose contract call ${CONTROLLER_ADDR} --recall-nonce \
          --pem=${OWNER} \
          --proxy=${PROXY} --chain=${CHAIN} \
          --gas-limit=100000000 \
          --function="addPlatforms" \
          --arguments "str:ashswap" erd1qqqqqqqqqqqqqpgqjwu8h7e2hvegj8sy9pxrl8rtvejlw7n9q33sg0t6gl 100 \
          --send || return 
}

setLiquidityBuffer() {
    mxpy --verbose contract call ${CONTROLLER_ADDR} --recall-nonce \
          --pem=${OWNER} \
          --proxy=${PROXY} --chain=${CHAIN} \
          --gas-limit=100000000 \
          --function="setLiquidityBuffer" \
          --arguments 100000000 \
          --send || return 
}

setFeesAddress() {
    mxpy --verbose contract call ${CONTROLLER_ADDR} --recall-nonce \
          --pem=${OWNER} \
          --proxy=${PROXY} --chain=${CHAIN} \
          --gas-limit=100000000 \
          --function="setFeesAddress" \
          --arguments erd1nnvazpzhan2a54xmmfrpdm6ltau0re333jfj9hhuu77kp0qnf53s9ayhdw \
          --send || return 
}

setDepositFees() {
    mxpy --verbose contract call ${CONTROLLER_ADDR} --recall-nonce \
          --pem=${OWNER} \
          --proxy=${PROXY} --chain=${CHAIN} \
          --gas-limit=100000000 \
          --function="setDepositFees" \
          --arguments 0 0 \
          --send || return 
}

getRewardsPerShare() {
    mxpy --verbose contract query ${CONTROLLER_ADDR} --function="getRewardsPerShare" --proxy=${PROXY} 
}

getControllerAddress() {
    mxpy --verbose contract query ${VAULT_ADDR} --function="getControllerAddress" --proxy=${PROXY} 
}