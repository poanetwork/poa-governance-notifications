use std::io::stderr;

use slog::{Drain, Logger};
use slog_term::{FullFormat, PlainSyncDecorator};
use web3::types::BlockNumber;

use error::Error;
use notify::Notification;

lazy_static! {
    pub static ref LOGGER: Logger = {
        let log_decorator = PlainSyncDecorator::new(stderr());
        let drain = FullFormat::new(log_decorator).build().fuse();
        Logger::root(drain, o!())
    };
}

pub fn log_ctrlc() {
    warn!(LOGGER, "recieved ctrl-c signal, gracefully shutting down...");
}

pub fn log_no_email_recipients_configured() {
    warn!(LOGGER, "email notifications are enabled, but there are no email recipients");
}

pub fn log_reached_notification_limit(notification_limit: u64) {
    warn!(
        LOGGER,
        "reached notification limit, gracefully shutting down...";
        "limit" => notification_limit
    );
}

pub fn log_finished_block_window(start: BlockNumber, stop: BlockNumber) {
    let block_range = format!("{:?}...{:?}", start, stop);
    info!(LOGGER, "finished checking blocks"; "block_range" => block_range);
}

pub fn log_notification(notif: &Notification) {
    let log = notif.log();
    info!(
        LOGGER,
        "governance notification";
        "ballot" => format!("{:?}", log.ballot_type),
        "ballot_id" => format!("{}", log.ballot_id),
        "block_number" => format!("{}", log.block_number)
    );
}

pub fn log_notification_verbose(notif: &Notification) {
    info!(LOGGER, "governance notification\n{}", notif.email_text());
}

pub fn log_failed_to_build_email(e: Error) {
    warn!(LOGGER, "failed to build email"; "error" => format!("{:?}", e));
}

pub fn log_failed_to_send_email(recipient: &str, e: Error) {
    warn!(LOGGER, "failed to send email"; "recipient" => recipient, "error" => format!("{:?}", e));
}

pub fn log_email_sent(recipient: &str) {
    info!(LOGGER, "email sent"; "to" => recipient);
}
