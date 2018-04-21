use std::io::stderr;

use lettre::smtp;
use slog::{Drain, Logger};
use slog_term::{FullFormat, PlainSyncDecorator};

use notify::Notification;

lazy_static! {
    pub static ref LOGGER: Logger = {
        let log_decorator = PlainSyncDecorator::new(stderr());
        let drain = FullFormat::new(log_decorator).build().fuse();
        Logger::root(drain, o!())
    };
}

pub fn log_notification(notif: &Notification) {
    info!(LOGGER, "notification"; "data" => format!("{:#?}", notif));
}

pub fn log_email_sent(email: &str) {
    info!(LOGGER, "email sent"; "to" => email);
}

pub fn log_email_failed(email: &str, error: smtp::error::Error) {
    warn!(
        LOGGER,
        "email failed";
        "to" => email,
        "error" => format!("{}", error)
    );
}
