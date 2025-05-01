use goose::prelude::*;
use rand::Rng;

// static PATH: &str = "/loadtest/84532";
static PATH: &str = "/1";

#[tokio::main]
async fn main() -> Result<(), GooseError> {
    GooseAttack::initialize()?
        .register_scenario(
            scenario!("basic")
                .register_transaction(transaction!(eth_block_number).set_name("eth_blockNumber"))
                .register_transaction(transaction!(eth_get_balance).set_name("eth_getBalance"))
                .register_transaction(
                    transaction!(eth_get_block_by_number).set_name("eth_getBlockByNumber"),
                ), // .register_transaction(transaction!(unknown_method).set_name("unknown_method")),
        )
        // .register_scenario(scenario!("random").register_transaction(
        //     transaction!(eth_block_by_number_random).set_name("eth_blockByNumberRandom"),
        // ))
        .execute()
        .await?;

    Ok(())
}

async fn eth_block_by_number_random(user: &mut GooseUser) -> TransactionResult {
    let random_number_hex = {
        let mut rng = rand::rng();
        let random_number = rng.random_range(0..100000);
        format!("0x{:x}", random_number)
    };
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_getBlockByNumber",
        "params": [random_number_hex, false],
        "id": 1
    });

    let _response = user.post_json(PATH, &request).await?;
    Ok(())
}

/// Get the current block number
async fn eth_block_number(user: &mut GooseUser) -> TransactionResult {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_blockNumber",
        "params": [],
        "id": 1
    });

    let _response = user.post_json(PATH, &request).await?;
    Ok(())
}

async fn eth_get_block_by_number(user: &mut GooseUser) -> TransactionResult {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_getBlockByNumber",
        "params": ["latest", false],
        "id": 1
    });

    let _response = user.post_json(PATH, &request).await?;
    Ok(())
}

/// Get the balance of a random address
async fn eth_get_balance(user: &mut GooseUser) -> TransactionResult {
    // Using a random address for testing
    let address = "0x742d35Cc6634C0532925a3b844Bc454e4438f44e";
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_getBalance",
        "params": [address, "latest"],
        "id": 1
    });

    let _response = user.post_json(PATH, &request).await?;
    Ok(())
}

async fn unknown_method(user: &mut GooseUser) -> TransactionResult {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "unknown_method",
        "params": [],
        "id": 1
    });

    let _response = user.post_json(PATH, &request).await?;
    Ok(())
}
