extern crate chrono;
extern crate clap;
extern crate dotenv;
extern crate ethabi;
extern crate ethereum_types;
extern crate hex;
extern crate jsonrpc_core;
#[macro_use] extern crate lazy_static;
extern crate lettre;
extern crate lettre_email;
extern crate native_tls;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate slog;
extern crate slog_term;
extern crate web3;

mod cli;
mod config;
mod logging;
mod notify;
mod rpc;
mod utils;

use config::Config;
use notify::Notifier;
use rpc::{BlockWindows, RpcClient};

fn main() {
    let config = Config::load();
    let client = RpcClient::new(&config.endpoint);
    let block_windows = BlockWindows::new(&client, config.avg_block_time);
    let mut notifier = Notifier::new(&config).unwrap();

    for (start, stop) in block_windows {
        for contract in &config.contracts {
            let ballot_created_logs = client
                .get_ballot_created_logs(contract, start, stop)
                .unwrap();

            for log in &ballot_created_logs {
                let voting_data = client.get_voting_state(contract, log.ballot_id).unwrap();
                let notif = notifier.build_notification(log, &voting_data);
                notifier.notify_validators(&notif);
            }
        }
    }
}
