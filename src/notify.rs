use chrono::{DateTime, Utc};
use ethereum_types::Address;
use lettre::{EmailTransport, SmtpTransport};
use lettre::smtp::{self, ConnectionReuseParameters};
use lettre::smtp::authentication::{Credentials, Mechanism};
use lettre_email::{self, Email, EmailBuilder};

use config::{Config, ContractType, Network, Validator};
use logging::{log_email_failed, log_email_sent, log_notification};
use rpc::{BallotCreatedLog, BallotType, KeyType, VotingData};

type BuildEmailResult = Result<Email, lettre_email::error::Error>;

#[derive(Debug)]
pub enum Notification {
    Keys(KeysNotification),
    Threshold(ThresholdNotification),
    Proxy(ProxyNotification)
}

#[derive(Debug)]
pub struct KeysNotification {
    pub network: Network,
    pub endpoint: String,
    pub block_number: u64,
    pub contract_type: ContractType,
    pub ballot_type: BallotType,
    pub ballot_id: u64,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub memo: String,
    pub affected_key: Address,
    pub affected_key_type: KeyType
}

#[derive(Debug)]
pub struct ThresholdNotification {
    pub network: Network,
    pub endpoint: String,
    pub block_number: u64,
    pub contract_type: ContractType,
    pub ballot_type: BallotType,
    pub ballot_id: u64,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub memo: String,
    pub proposed_value: u64
}

#[derive(Debug)]
pub struct ProxyNotification {
    pub network: Network,
    pub endpoint: String,
    pub block_number: u64,
    pub contract_type: ContractType,
    pub ballot_type: BallotType,
    pub ballot_id: u64,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub memo: String,
    pub proposed_value: Address
}

impl Notification {
    fn new(config: &Config, log: &BallotCreatedLog, voting_data: &VotingData) -> Self {
        let network = config.network;
        let endpoint = config.endpoint.clone();

        let block_number = log.block_number;
        let ballot_type = log.ballot_type;
        let ballot_id = log.ballot_id;

        let start_time = voting_data.start_time();
        let end_time = voting_data.end_time();
        let memo = voting_data.memo();

        match *voting_data {
            VotingData::Keys(ref data) => {
                let contract_type = ContractType::Keys;
                let affected_key = data.affected_key;
                let affected_key_type = data.affected_key_type;
                let notification = KeysNotification {
                    network, endpoint, block_number,
                    contract_type, ballot_type, ballot_id,
                    start_time, end_time, memo,
                    affected_key, affected_key_type
                };
                Notification::Keys(notification)
            },
            VotingData::Threshold(ref data) => {
                let contract_type = ContractType::Threshold;
                let proposed_value = data.proposed_value;
                let notification = ThresholdNotification {
                    network, endpoint, block_number,
                    contract_type, ballot_type,
                    ballot_id, start_time, end_time,
                    memo, proposed_value
                };
                Notification::Threshold(notification)
            },
            VotingData::Proxy(ref data) => {
                let contract_type = ContractType::Proxy;
                let proposed_value = data.proposed_value;
                let notification = ProxyNotification {
                    network, endpoint, block_number,
                    contract_type, ballot_type,
                    ballot_id, start_time, end_time,
                    memo, proposed_value
                };
                Notification::Proxy(notification)
            }
        }
    }
}

pub struct Notifier<'a> {
    config: &'a Config,
    mailer: Option<SmtpTransport>
}

impl<'a> Notifier<'a> {
    pub fn new(config: &'a Config) -> Result<Self, smtp::error::Error> {
        let mailer = if config.send_email_notifications {
            let creds = Credentials::new(
                config.smtp_username.clone(),
                config.smtp_password.clone()
            );
            let mailer = SmtpTransport::simple_builder(&config.smtp_host_domain)?
                .connection_reuse(ConnectionReuseParameters::ReuseUnlimited)
                .authentication_mechanism(Mechanism::Plain)
                .credentials(creds)
                .build();
            Some(mailer)
        } else {
            None
        };

        Ok(Notifier { config, mailer })
    }

    pub fn build_notification(&self, log: &BallotCreatedLog, voting_data: &VotingData) -> Notification {
        Notification::new(&self.config, log, voting_data)
    }

    pub fn notify_validators(&mut self, notif: &Notification) {
        log_notification(notif);
        for validator in &self.config.validators {
            if self.config.send_email_notifications {
                let email = self.build_email(validator, notif).unwrap();
                if let Some(ref mut mailer) = self.mailer {
                    match mailer.send(&email) {
                        Ok(_) => log_email_sent(&validator.email),
                        Err(e) => log_email_failed(&validator.email, e)
                    };
                }
            }

            if self.config.send_push_notifications {
                println!("Push Notifications not yet implemented.");
            }
        }
    }

    fn build_email(&self, validator: &Validator, notif: &Notification) -> BuildEmailResult {
        let body = match *notif {
            Notification::Keys(ref inner) => format!("{:#?}\n", inner),
            Notification::Threshold(ref inner) => format!("{:#?}\n", inner),
            Notification::Proxy(ref inner) => format!("{:#?}\n", inner)
        };
        EmailBuilder::new()
            .to(validator.email.as_str())
            .from(self.config.outgoing_email.as_str())
            .subject("POA Network Governance Notification")
            .text(body)
            .build()
    }
}

impl<'a> Drop for Notifier<'a> {
    fn drop(&mut self) {
        if let Some(ref mut mailer) = self.mailer {
            mailer.close();
        }
    }
}
