use novax::ashswapplatformcontract::ashswapplatformcontract::AshSwapPlatformContractContract;
use novax::code::DeployData;
use novax::executor::NetworkExecutor;
use novax::{CodeMetadata, Wallet};
use novax::ashswapholdercontract::ashswapholdercontract::AshSwapHolderContractContract;

const GATEWAY_URL: &str = "https://devnet-gateway.multiversx.com";
const WASM_PATH: &str = "../output/platform.wasm";
const CONTROLLER_ADDRESS: &str = "erd1qqqqqqqqqqqqqpgqzzlkdyqvgkvwmapyupauckfmy6krwh4ef53sf50atn";

#[tokio::main]
async fn main() {
    deploy().await;
}

fn get_wallet() -> Wallet {
    Wallet::from_pem_file(&*std::env::var("WALLET_PATH").unwrap()).unwrap()
}

async fn deploy() {
    let wallet = get_wallet();

    let deploy_data = DeployData {
        code: WASM_PATH,
        metadata: CodeMetadata::PAYABLE_BY_SC | CodeMetadata::UPGRADEABLE,
    };

    let (new_address, _) = AshSwapPlatformContractContract::deploy(
        deploy_data,
        &mut NetworkExecutor::new(GATEWAY_URL, &wallet),
        600_000_000,
        &CONTROLLER_ADDRESS.into(),
        &"erd1qqqqqqqqqqqqqpgqx99hh0w3zur8zxj9p4cs2fltg73ay4v7q33sfhzv8k".into(),
        &"erd1qqqqqqqqqqqqqpgqarwv448dk6phts2tlt6yjzuda7evw9p9q33scdw2vz".into(),
        &"USDC-8d4068".to_string()
    )
        .await
        .unwrap();

    AshSwapHolderContractContract::new(
        "erd1qqqqqqqqqqqqqpgqx99hh0w3zur8zxj9p4cs2fltg73ay4v7q33sfhzv8k"
    )
        .call(
            NetworkExecutor::new(GATEWAY_URL, &wallet),
            600_000_000
        )
        .whitelist_contract(
            &new_address
        )
        .await
        .expect("Failed to whitelist deployed platform on the ashswap holder");

    let ashswap_platform_contract = AshSwapPlatformContractContract::new(
        new_address.clone()
    );

    // add USDC USDT BUSD pool
    ashswap_platform_contract
        .clone()
        .call(
            NetworkExecutor::new(GATEWAY_URL, &wallet),
            600_000_000
        )
        .add_pool(
            &"erd1qqqqqqqqqqqqqpgq3sfgh89vurcdet6lwl4cysqddeyk0rqh2gesqpkk4e".into(),
            &"erd1qqqqqqqqqqqqqpgqpr6tklrup3ld2z59jurcq94z8668uuey2ges9thxd2".into(),
            &2
        )
        .await
        .expect("Failed to add USDC-USDT-BUSD pool");

    // add USDT HTM pool
    ashswap_platform_contract
        .call(
            NetworkExecutor::new(GATEWAY_URL, &wallet),
            600_000_000
        )
        .add_pool(
            &"erd1qqqqqqqqqqqqqpgqwqrrwk3npn4d26q5f0ltsang08q0fj8w2gesa4karx".into(),
            &"erd1qqqqqqqqqqqqqpgqn65rsy79a9pml86jptyhaucz7w32j7jw2gesglrlz2".into(),
            &1
        )
        .await
        .expect("Failed to add USDT-HTM pool");

    // enable ASH as swappable token
    AshSwapPlatformContractContract::new(
        new_address.clone()
    )
        .call(
            NetworkExecutor::new(GATEWAY_URL, &wallet),
            600_000_000
        )
        .set_swappable_token(
            &"ASH-4ce444".to_string(),
            &true
        )
        .await
        .expect("Failed to set ASH as swappable token");

    println!("Done! Deployed address: {}", new_address.to_bech32_string().unwrap())

}
