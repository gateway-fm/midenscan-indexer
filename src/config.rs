use miden_protocol::address::NetworkId;
use once_cell::sync::Lazy;

#[derive(Debug)]
pub struct Config {
    pub postgres_url: String,
    pub rpc_url: String,
    pub miden_network: NetworkId,
    pub http_port: u16,
}

fn load() -> Config {
    // dev convenience, load variables from a `.env` file if it exists.
    let _ = dotenvy::dotenv();

    let postgres_url = std::env::var("POSTGRES_URL").expect("POSTGRES_URL must be set");

    let rpc_url = std::env::var("RPC_URL").expect("RPC_URL must be set");

    let raw_network = std::env::var("MIDEN_NETWORK")
        .expect("MIDEN_NETWORK must be set")
        .to_lowercase();
    let miden_network = if raw_network == "mainnet" {
        NetworkId::Mainnet
    } else if raw_network == "testnet" {
        NetworkId::Testnet
    } else if raw_network == "devnet" {
        NetworkId::Devnet
    } else {
        panic!(
            "MIDEN_NETWORK must be `mainnet`, `testnet`, or `devnet` (got `{}`)",
            raw_network
        );
    };

    // optional, defaults to 8080
    let http_port = std::env::var("HTTP_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(8080);

    Config {
        postgres_url,
        rpc_url,
        miden_network,
        http_port,
    }
}
pub static CONFIG: Lazy<Config> = Lazy::new(load);
