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

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

use blockchain::BlockchainIter;
use cli::parse_cli;
use client::RpcClient;
use config::{Config, ContractVersion};
use error::{Error, Result};
use logger::Logger;
use notify::{Notification, Notifier};

lazy_static! {
    // Tracks whether or not the environment variables have been loaded from the .env file.
    static ref LOADED_ENV_FILE: AtomicBool = AtomicBool::new(false);
}

/// Attempts to load the .env file once at the start of the main process or at the start of the
/// tests. Panics if the .env file cannot be found or if it cannot be parsed (most likely it
/// contains invalid UTF-8 bytes).
fn load_env_file() {
    if !LOADED_ENV_FILE.load(Ordering::Relaxed) {
        match dotenv::dotenv() {
            Ok(_) => LOADED_ENV_FILE.store(true, Ordering::Relaxed),
            Err(dotenv::Error::Io(_)) => panic!("could not find .env file"),
            _ =>  panic!("could not parse .env file"),
        };
    }
}

/// Sets up ctrl-c to change the value of `poagov_is_running` from `true` to `false`. When
/// `poagov_is_running` changes to `false`, the `poagov` process begins to gracefully shut down.
/// The `AtomicBool` returned by this function is used to indicate whether or not the `poagov`
/// binary should continue running.
fn set_ctrlc_handler(logger: Arc<Mutex<Logger>>) -> Result<Arc<AtomicBool>> {
    let poagov_is_running = Arc::new(AtomicBool::new(true));
    let setup_res = {
        let poagov_is_running = poagov_is_running.clone();
        ctrlc::set_handler(move || {
            logger.lock().unwrap().log_ctrlc_pressed();
            poagov_is_running.store(false, Ordering::SeqCst);
        })
    };
    if let Err(e) = setup_res {
        Err(Error::CtrlcSetupError(e))
    } else {
        Ok(poagov_is_running)
    }
}

fn main() -> Result<()> {
    load_env_file();

    let cli = parse_cli();
    let config = Config::new(&cli)?;
    let logger = Arc::new(Mutex::new(Logger::new(&config)));
    let running = set_ctrlc_handler(logger.clone())?;
    let client = RpcClient::new(config.endpoint.clone());
    let blockchain_iter = BlockchainIter::new(&client, &config, running)?;
    let mut notifier = Notifier::new(&config, logger.clone())?;

    // If email notifications have been enabled but there are no email recipients configured, warn
    // the user.
    if config.email_notifications && config.email_recipients.is_empty() {
        logger.lock().unwrap().log_no_email_recipients_configured();
    }
    logger.lock().unwrap().log_starting_poagov();

    'blockchain_walker: for block_range_res in blockchain_iter {
        let (start_block, stop_block) = block_range_res?;
        let mut notifications = vec![];

        // For each contract that we are monitoring for governance events, get the ballot-created
        // events that fall within the current `BlockchainIter`'s block window, convert those
        // ballot-created logs to `Notification`s.
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

        // Sort the notifications by ascending block number.
        notifications.sort_unstable_by(|notif1, notif2| {
            notif1.log().block_number.cmp(&notif2.log().block_number)
        });

        // Notify the governance notifications recipients.
        for notification in notifications {
            notifier.notify(&notification);
            if notifier.reached_limit() {
                let limit = config.notification_limit.unwrap();
                logger.lock().unwrap().log_reached_notification_limit(limit);
                break 'blockchain_walker;
            }
        }

        logger.lock().unwrap().log_finished_block_window(start_block, stop_block);
    }

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::load_env_file;

    /// Loads the .env file once at the start of the tests.
    pub fn setup() {
        load_env_file();
    }
}
