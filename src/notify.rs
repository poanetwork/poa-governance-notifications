use lettre::{SendableEmail, Transport};
use lettre::smtp::{ClientSecurity, ConnectionReuseParameters, SmtpClient, SmtpTransport};
use lettre::smtp::authentication::{Credentials, Mechanism};
use lettre::smtp::client::net::ClientTlsParameters;
use lettre_email::{Email, EmailBuilder};
use native_tls::TlsConnector;

use config::Config;
use error::{Error, Result};
use logger::{
    log_email_sent, log_failed_to_build_email, log_failed_to_send_email, log_notification,
    log_notification_verbose,
};
use response::common::BallotCreatedLog;
use response::v1::VotingState;
use response::v2::BallotInfo;

#[derive(Clone, Debug)]
pub enum Notification<'a> {
    VotingState {
        config: &'a Config,
        log: BallotCreatedLog,
        voting_state: VotingState,
    },
    BallotInfo {
        config: &'a Config,
        log: BallotCreatedLog,
        ballot_info: BallotInfo,
    },
}

impl<'a> Notification<'a> {
    pub fn from_voting_state(
        config: &'a Config,
        log: BallotCreatedLog,
        voting_state: VotingState,
    ) -> Self
    {
        Notification::VotingState { config, log, voting_state }
    }
    
    pub fn from_ballot_info(
        config: &'a Config,
        log: BallotCreatedLog,
        ballot_info: BallotInfo,
    ) -> Self
    {
        Notification::BallotInfo { config, log, ballot_info }
    }

    pub fn email_text(&self) -> String {
        format!(
            "Network: {:?}\n\
            RPC Endpoint: {}\n\
            Block Number: {}\n\
            Contract: {}\n\
            Version: {:?}\n\
            Ballot ID: {}\n\
            {}\n",
            self.config().network,
            self.config().endpoint,
            self.log().block_number,
            self.contract_name(),
            self.config().version,
            self.log().ballot_id,
            self.email_body(),
        )
    }

    fn config(&self) -> &Config {
        match self {
            Notification::VotingState { config, .. } => config,
            Notification::BallotInfo { config, .. } => config,
        }
    }

    pub fn log(&self) -> &BallotCreatedLog {
        match self {
            Notification::VotingState { log, .. } => log,
            Notification::BallotInfo { log, .. } => log,
        }
    }
    
    fn contract_name(&self) -> String {
        match self {
            Notification::VotingState { voting_state, .. } => voting_state.contract_name(),
            Notification::BallotInfo { ballot_info, .. } => ballot_info.contract_name(),
        }
    }

    fn email_body(&self) -> String {
        match self {
            Notification::VotingState { voting_state, .. } => voting_state.email_text(),
            Notification::BallotInfo { ballot_info, .. } => ballot_info.email_text(),
        }
    }
}

pub struct Notifier<'a> {
    config: &'a Config,
    emailer: Option<SmtpTransport>,
}

impl<'a> Notifier<'a> {
    pub fn new(config: &'a Config) -> Result<Self> {
        let emailer = if config.email_notifications {
            let domain = config.smtp_host_domain.clone().unwrap();
            let port = config.smtp_port.unwrap();
            let addr = (domain.as_str(), port);
            let security = {
                let tls = TlsConnector::new().map_err(|e| Error::FailedToBuildTls(e))?;
                let smtp_security_setup = ClientTlsParameters::new(domain.clone(), tls);
                ClientSecurity::Required(smtp_security_setup)
            };
            let creds = Credentials::new(
                config.smtp_username.clone().unwrap(),
                config.smtp_password.clone().unwrap(),
            );
            let smtp = SmtpClient::new(addr, security)
                .map_err(|e| Error::FailedToResolveSmtpHostDomain(e))?
                .connection_reuse(ConnectionReuseParameters::ReuseUnlimited)
                .authentication_mechanism(Mechanism::Plain)
                .credentials(creds)
                .transport();
            Some(smtp)
        } else {
            None
        };
        Ok(Notifier { config, emailer })
    }

    pub fn notify(&mut self, notif: &Notification) {
        if self.config.verbose_logs {
            log_notification_verbose(notif);
        } else {
            log_notification(notif);
        }
        if self.config.email_notifications {
            for recipient in self.config.email_recipients.iter() {
                let email: SendableEmail = match self.build_email(notif, recipient) {
                    Ok(email) => email.into(),
                    Err(e) => {
                        log_failed_to_build_email(e);
                        continue;
                    },
                };
                if let Err(e) = self.send_email(email) {
                    log_failed_to_send_email(recipient, e);
                } else {
                    log_email_sent(recipient);
                }
            }
        }
    }

    fn build_email(&self, notif: &Notification, recipient: &str) -> Result<Email> {
        let outgoing_email = self.config.outgoing_email_addr.clone().unwrap();
        EmailBuilder::new()
            .to(recipient)
            .from(outgoing_email.as_str())
            .subject("POA Network Governance Notification")
            .text(notif.email_text())
            .build()
            .map_err(|e| Error::FailedToBuildEmail(e))
    }

    fn send_email(&mut self, email: SendableEmail) -> Result<()> {
        if let Some(ref mut emailer) = self.emailer {
            match emailer.send(email) {
                Ok(_response) => Ok(()),
                Err(e) => Err(Error::FailedToSendEmail(e)),
            }
        } else {
            unreachable!("Attempted to send email without SMTP client setup");
        }
    }
}
