PROXY="https://devnet-gateway.multiversx.com"
CHAIN="D"
OWNER="swallet.pem"
CONTROLLER="controller/output/controller.wasm"

SC_ADDRESS="erd1qqqqqqqqqqqqqpgqxxz6h94us93hyj0wrzrd5y8nua24ema4ehjq8s5n3n"

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

upgradeController() {
    mxpy --verbose contract upgrade ${SC_ADDRESS} --bytecode="$CONTROLLER" --recall-nonce \
    --pem=${OWNER} \
    --gas-limit=599000000 \
    --proxy=${PROXY} --chain=${CHAIN} \
    --metadata-payable-by-sc \
    --arguments $USDC_TOKEN_ID $PHASE $MIN_UNBOND_EPOCHS $WITHDRAW_FEES_PERC \
    --send --outfile="deploy-devnet.interaction.json" || return

    echo "Smart contract upgraded address: ${ADDRESS}"
}

registerSavingsToken() {
    NAME="str:AutoscaleSavingsUSDC"
    TICKER="str:ASUSDC"
    DECIMALS=6

    mxpy --verbose contract call ${SC_ADDRESS} --recall-nonce \
          --pem=${OWNER} \
          --proxy=${PROXY} --chain=${CHAIN} \
          --gas-limit=100000000 \
          --value=50000000000000000 \
          --function="registerSavingsToken" \
          --arguments $NAME $TICKER $DECIMALS \
          --send || return
}

registerUnbondToken() {
    NAME="str:AutoscaleSavingsUnbondUSDC"
    TICKER="str:ASUUSDC"
    DECIMALS=6

    mxpy --verbose contract call ${SC_ADDRESS} --recall-nonce \
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

    mxpy --verbose contract call ${SC_ADDRESS} --recall-nonce \
          --pem=${OWNER} \
          --proxy=${PROXY} --chain=${CHAIN} \
          --gas-limit=100000000 \
          --function="setDepositFees" \
          --arguments $phase $fees_perc \
          --send || return
}

setRewardsPerSharePerBlock() {
    new_rewards_per_share_per_bloc=100

    mxpy --verbose contract call ${SC_ADDRESS} --recall-nonce \
          --pem=${OWNER} \
          --proxy=${PROXY} --chain=${CHAIN} \
          --gas-limit=100000000 \
          --function="setRewardsPerSharePerBlock" \
          --arguments $new_rewards_per_share_per_bloc \
          --send || return
}

setProduceRewardsEnabled() {
    bool=1

    mxpy --verbose contract call ${SC_ADDRESS} --recall-nonce \
          --pem=${OWNER} \
          --proxy=${PROXY} --chain=${CHAIN} \
          --gas-limit=100000000 \
          --function="setProduceRewardsEnabled" \
          --arguments $bool \
          --send || return
}