extern crate chrono;
extern crate clap;
extern crate ctrlc;
extern crate dotenv;
extern crate ethabi;
extern crate ethereum_types;
extern crate failure;
extern crate hex;
extern crate jsonrpc_core;
#[macro_use]
extern crate lazy_static;
extern crate lettre;
extern crate lettre_email;
extern crate native_tls;
extern crate reqwest;
extern crate serde_json;
#[macro_use]
extern crate slog;
extern crate slog_term;
extern crate web3;

mod blockchain;
mod cli;
mod client;
mod config;
mod error;
mod logger;
mod notify;
mod response;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use blockchain::BlockchainIter;
use cli::Cli;
use client::RpcClient;
use config::{Config, ContractVersion};
use error::{Error, Result};
use logger::{log_ctrlc, log_finished_block_window, log_reached_notification_limit};
use notify::{Notification, Notifier};

fn load_dotenv_file() -> Result<()> {
    if let Err(e) = dotenv::dotenv() {
        if let dotenv::Error::Io(_) = e {
            Err(Error::EnvFileNotFound(e))
        } else {
            Err(Error::FailedToParseEnvFile(e))
        }
    } else {
        Ok(())
    }
}

fn set_ctrlc_handler() -> Result<Arc<AtomicBool>> {
    let running = Arc::new(AtomicBool::new(true));
    {
        let running = running.clone();
        ctrlc::set_handler(move || {
            log_ctrlc();
            running.store(false, Ordering::SeqCst);
        }).map_err(|e| Error::CtrlcError(e))?;
    }
    Ok(running)
}

fn main() -> Result<()> {
    load_dotenv_file()?;
    let cli = Cli::parse();
    let config = Config::new(&cli)?;
    let running = set_ctrlc_handler()?;
    let client = RpcClient::new(config.endpoint.clone());
    let mut notifier = Notifier::new(&config)?;
    let mut notification_count = 0;
    'main_loop: for iter_res in BlockchainIter::new(&client, &config, running)? {
        let (start_block, stop_block) = iter_res?;
        let mut notifications = vec![];
        for contract in config.contracts.iter() {
            let ballot_created_logs = client.get_ballot_created_logs(
                contract,
                start_block,
                stop_block,
            )?;
            for log in ballot_created_logs.into_iter() { 
                let notification = if contract.version == ContractVersion::V1 {
                    let voting_state = client.get_voting_state(contract, log.ballot_id)?;
                    Notification::from_voting_state(&config, log, voting_state)
                } else {
                    let ballot_info = client.get_ballot_info(contract, log.ballot_id)?;
                    Notification::from_ballot_info(&config, log, ballot_info)
                };
                notifications.push(notification);
            }
        }
        notifications.sort_unstable_by(|notif1, notif2| {
            notif1.log().block_number.cmp(&notif2.log().block_number)
        });
        for notification in notifications.iter() {
            notifier.notify(notification);
            if let Some(notification_limit) = config.notification_limit {
                notification_count += 1;
                if notification_count >= notification_limit {
                    log_reached_notification_limit(notification_limit);
                    break 'main_loop;
                }
            }
        }
        log_finished_block_window(start_block, stop_block);
    }
    Ok(())
}
