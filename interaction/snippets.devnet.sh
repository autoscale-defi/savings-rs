PROXY="https://devnet-gateway.multiversx.com"
CHAIN="D"
OWNER="swallet.pem"

CONTROLLER="controller/output/controller.wasm"
VAULT="vault/output/vault.wasm"

CONTROLLER_ADDR="erd1qqqqqqqqqqqqqpgqfj8jxtk8crkjq0whsyfl58pkpa8vljsxehjq2p8ems"
VAULT_ADDR="erd1qqqqqqqqqqqqqpgqyyvtfscnszfpm3gs0e6jgr8vcj8jcym4ehjq2fz4xc"

USDC_TOKEN_ID="str:USDC-8d4068"
PHASE=0
MIN_UNBOND_EPOCHS=7
WITHDRAW_FEES_PERC=10

deployController() {
  mxpy --verbose contract deploy --bytecode="$CONTROLLER" --recall-nonce \
        --pem=$OWNER \
        --gas-limit=599000000 \
        --proxy=$PROXY --chain=$CHAIN \
        --metadata-payable-by-sc \
        --arguments $USDC_TOKEN_ID $PHASE $MIN_UNBOND_EPOCHS $WITHDRAW_FEES_PERC \
        --outfile="deploy-devnet.interaction.json" --send || return
}

deployVault() {
  mxpy --verbose contract deploy --bytecode="$VAULT" --recall-nonce \
        --pem=$OWNER \
        --gas-limit=599000000 \
        --proxy=$PROXY --chain=$CHAIN \
        --metadata-payable-by-sc \
        --arguments $USDC_TOKEN_ID $MY_ADDR \
        --outfile="deploy-devnet.interaction.json" --send || return
}

upgradeController() {
    mxpy --verbose contract upgrade ${CONTROLLER_ADDR} --bytecode="$CONTROLLER" --recall-nonce \
    --pem=${OWNER} \
    --gas-limit=599000000 \
    --proxy=${PROXY} --chain=${CHAIN} \
    --metadata-payable-by-sc \
    --arguments $USDC_TOKEN_ID $PHASE $MIN_UNBOND_EPOCHS $WITHDRAW_FEES_PERC \
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

setDepositFeesInAccPhase() {
    phase=0
    fees_perc=1

    mxpy --verbose contract call ${CONTROLLER_ADDR} --recall-nonce \
          --pem=${OWNER} \
          --proxy=${PROXY} --chain=${CHAIN} \
          --gas-limit=100000000 \
          --function="setDepositFees" \
          --arguments $phase $fees_perc \
          --send || return
}

setRewardsPerSharePerBlock() {
    new_rewards_per_share_per_bloc=10

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

getRewardsPerShare() {
    mxpy --verbose contract query ${CONTROLLER_ADDR} --function="getRewardsPerShare" --proxy=${PROXY} 
}

getControllerAddress() {
    mxpy --verbose contract query ${VAULT_ADDR} --function="getControllerAddress" --proxy=${PROXY} 
}