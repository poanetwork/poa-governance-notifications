use std::env;
use std::fmt::{self, Debug, Formatter};
use std::fs::File;
use std::str::FromStr as _FromStr;

use ethabi::{Address, Contract, Event, Function};

use crate::cli::Cli;
use crate::error::{Error, Result};
use crate::response::common::BallotType;

const DEFAULT_BLOCK_TIME_SECS: u64 = 30;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Network {
    Core,
    Sokol,
}

impl Network {
    fn uppercase(&self) -> &str {
        match self {
            Network::Core => "CORE",
            Network::Sokol => "SOKOL",
        }
    }
}

/// Note that the `Emission` contract is V2 only.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ContractType {
    Keys,
    Threshold,
    Proxy,
    Emission,
}

impl From<BallotType> for ContractType {
    fn from(ballot_type: BallotType) -> Self {
        match ballot_type {
            BallotType::InvalidKey => ContractType::Keys,
            BallotType::AddKey => ContractType::Keys,
            BallotType::RemoveKey => ContractType::Keys,
            BallotType::SwapKey => ContractType::Keys,
            BallotType::Threshold => ContractType::Threshold,
            BallotType::Proxy => ContractType::Proxy,
            BallotType::Emission => ContractType::Emission,
        }
    }
}

impl ContractType {
    fn uppercase(&self) -> &str {
        match self {
            ContractType::Keys => "KEYS",
            ContractType::Threshold => "THRESHOLD",
            ContractType::Proxy => "PROXY",
            ContractType::Emission => "EMISSION_FUNDS",
        }
    }

    fn abi_file_name(&self) -> &str {
        match self {
            ContractType::Keys => "VotingToChangeKeys.abi.json",
            ContractType::Threshold => "VotingToChangeMinThreshold.abi.json",
            ContractType::Proxy => "VotingToChangeProxyAddress.abi.json",
            ContractType::Emission => "VotingToManageEmissionFunds.abi.json",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ContractVersion {
    V1,
    V2,
}

impl ContractVersion {
    fn lowercase(&self) -> &str {
        match self {
            ContractVersion::V1 => "v1",
            ContractVersion::V2 => "v2",
        }
    }
}

#[derive(Clone)]
pub struct PoaContract {
    pub kind: ContractType,
    pub version: ContractVersion,
    pub addr: Address,
    pub abi: Contract,
}

impl Debug for PoaContract {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("PoaContract")
            .field("kind", &self.kind)
            .field("addr", &self.addr)
            .field("abi", &"<ethabi::Contract>")
            .finish()
    }
}

impl PoaContract {
    pub fn read(
        contract_type: ContractType,
        network: Network,
        version: ContractVersion,
    ) -> Result<Self>
    {
        if contract_type == ContractType::Emission && version == ContractVersion::V1 {
            return Err(Error::EmissionFundsV1ContractDoesNotExist);
        }

        let addr_env_var = format!(
            "{}_CONTRACT_ADDRESS_{}_{:?}",
            contract_type.uppercase(),
            network.uppercase(),
            version,
        );

        let addr = if let Ok(s) = env::var(&addr_env_var) {
            match Address::from_str(s.trim_left_matches("0x")) {
                Ok(addr) => addr,
                Err(_) => return Err(Error::InvalidContractAddr(s.into())),
            }
        } else {
            return Err(Error::MissingEnvVar(addr_env_var));
        };

        let abi_path = format!(
            "abis/{}/{}",
            version.lowercase(),
            contract_type.abi_file_name(),
        );

        let abi_file = File::open(&abi_path)
            .map_err(|_| Error::MissingAbiFile(abi_path.clone()))?;

        let abi = Contract::load(&abi_file)
            .map_err(|_| Error::InvalidAbi(abi_path))?;

        let contract = PoaContract { kind: contract_type, version, addr, abi };
        Ok(contract)
    }

    pub fn event(&self, event: &str) -> Event {
        self.abi.event(event).unwrap().clone()
    }

    pub fn function(&self, function: &str) -> Function {
        self.abi.function(function).unwrap().clone()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StartBlock {
    Earliest,
    Latest,
    Number(u64),
    Tail(u64),
}

#[derive(Clone, Debug)]
pub struct Config {
    pub network: Network,
    pub endpoint: String,
    pub version: ContractVersion,
    pub contracts: Vec<PoaContract>,
    pub start_block: StartBlock,
    pub block_time: u64,
    pub email_notifications: bool,
    pub email_recipients: Vec<String>,
    pub smtp_host_domain: Option<String>,
    pub smtp_port: Option<u16>,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub outgoing_email_addr: Option<String>,
    pub notification_limit: Option<usize>,
    pub log_emails: bool,
    pub log_to_file: bool,
}

impl Config {
    pub fn new(cli: &Cli) -> Result<Self> {
        let network = if cli.core() == cli.sokol() {
            return Err(Error::MustSpecifyOneCliArgument("`--core` or `--sokol`".into()));
        } else if cli.core() {
            Network::Core
        } else {
            Network::Sokol
        };

        let endpoint_env_var = format!("{}_RPC_ENDPOINT", network.uppercase());
        let endpoint = env::var(&endpoint_env_var)
            .map_err(|_| Error::MissingEnvVar(endpoint_env_var))?;

        let version = if cli.v1() == cli.v2(){
            return Err(Error::MustSpecifyOneCliArgument("`--v1` or `--v2`".into()));
        } else if cli.v1(){
            ContractVersion::V1
        } else {
            ContractVersion::V2
        };

        let mut contracts = vec![];
        if cli.keys() {
            let keys = PoaContract::read(
                ContractType::Keys,
                network,
                version
            )?;
            contracts.push(keys);
        }
        if cli.threshold() {
            let threshold = PoaContract::read(
                ContractType::Threshold,
                network,
                version
            )?;
            contracts.push(threshold);
        }
        if cli.proxy() {
            let proxy = PoaContract::read(
                ContractType::Proxy,
                network,
                version
            )?;
            contracts.push(proxy);
        }
        if cli.emission() {
            let emission_funds = PoaContract::read(
                ContractType::Emission,
                network,
                version,
            )?;
            contracts.push(emission_funds);
        }
        if contracts.is_empty() {
            return Err(Error::MustSpecifyAtLeastOneCliArgument(
                "`--keys`, `--threshold`, `--proxy`, `--emission`".into()
            ));
        }

        let start_block = if cli.multiple_start_blocks_specified() {
            return Err(Error::MustSpecifyOneCliArgument(
                "`--earliest` or `--latest` or `--start-block` or `--tail`".into()
            ));
        } else if cli.earliest() {
            StartBlock::Earliest
        } else if cli.latest() {
            StartBlock::Latest
        } else if let Some(s) = cli.start_block() {
            let block_number = s.parse().map_err(|_| Error::InvalidStartBlock(s.into()))?;
            StartBlock::Number(block_number)
        } else if let Some(s) = cli.tail() {
            let tail = s.parse().map_err(|_| Error::InvalidTail(s.into()))?;
            StartBlock::Tail(tail)
        } else {
            unreachable!();
        };

        let block_time = if let Some(s) = cli.block_time() {
            s.parse().map_err(|_| Error::InvalidBlockTime(s.into()))?
        } else {
            DEFAULT_BLOCK_TIME_SECS
        };

        let email_notifications = cli.email();

        let email_recipients: Vec<String> = env::var("EMAIL_RECIPIENTS")
            .map_err(|_| Error::MissingEnvVar("EMAIL_RECIPIENTS".into()))?
            .split(',')
            .filter_map(|s| {
                if s.is_empty() {
                    None
                } else {
                    Some(s.into())
                }
            })
            .collect();

        let smtp_host_domain = if email_notifications {
            let host = env::var("SMTP_HOST_DOMAIN")
                .map_err(|_| Error::MissingEnvVar("SMTP_HOST_DOMAIN".into()))?;
            Some(host)
        } else {
            None
        };

        let smtp_port = if email_notifications {
            if let Ok(s) = env::var("SMTP_PORT") {
                let port = s.parse().map_err(|_| Error::InvalidSmtpPort(s.into()))?;
                Some(port)
            } else {
                return Err(Error::MissingEnvVar("SMTP_PORT".into()));
            }
        } else {
            None
        };

        let smtp_username = if email_notifications {
            let username = env::var("SMTP_USERNAME")
                .map_err(|_| Error::MissingEnvVar("SMTP_USERNAME".into()))?;
            Some(username)
        } else {
            None
        };

        let smtp_password = if email_notifications {
            let password = env::var("SMTP_PASSWORD")
                .map_err(|_| Error::MissingEnvVar("SMTP_PASSWORD".into()))?;
            Some(password)
        } else {
            None
        };

        let outgoing_email_addr = if email_notifications {
            let email_addr = env::var("OUTGOING_EMAIL_ADDRESS")
                .map_err(|_| Error::MissingEnvVar("OUTGOING_EMAIL_ADDRESS".into()))?;
            Some(email_addr)
        } else {
            None
        };

        let notification_limit = if let Some(s) = cli.notification_limit() {
            let limit = s.parse().map_err(|_| Error::InvalidNotificationLimit(s.into()))?;
            Some(limit)
        } else {
            None
        };

        let log_emails = cli.log_emails();
        let log_to_file = cli.log_to_file();

        let config = Config {
            network,
            endpoint,
            version,
            contracts,
            start_block,
            block_time,
            email_notifications,
            email_recipients,
            smtp_host_domain,
            smtp_port,
            smtp_username,
            smtp_password,
            outgoing_email_addr,
            notification_limit,
            log_emails,
            log_to_file,
        };
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::super::tests::setup;
    use super::{ContractType, ContractVersion, PoaContract, Network};

    const CONTRACT_TYPES: [ContractType; 4] = [
            ContractType::Keys,
            ContractType::Threshold,
            ContractType::Proxy,
            ContractType::Emission,
    ];
    const NETWORKS: [Network; 2] = [Network::Sokol, Network::Core];
    const VERSIONS: [ContractVersion; 2] = [ContractVersion::V1, ContractVersion::V2];

    #[test]
    fn test_env_file_integrity() {
        setup();
        for network in NETWORKS.iter() {
            let env_var = format!("{}_RPC_ENDPOINT", network.uppercase());
            assert!(env::var(&env_var).is_ok());
            for contract_type in CONTRACT_TYPES.iter() {
                for version in VERSIONS.iter() {
                    if *contract_type == ContractType::Emission && *version == ContractVersion::V1 {
                        continue;
                    }
                    let env_var = format!(
                        "{}_CONTRACT_ADDRESS_{}_{:?}",
                        contract_type.uppercase(),
                        network.uppercase(),
                        version,
                    );
                    assert!(env::var(&env_var).is_ok());
                }
            }
        }
    }

    #[test]
    fn test_load_contract_abis() {
        setup();
        for contract_type in CONTRACT_TYPES.iter() {
            for version in VERSIONS.iter() {
                for network in NETWORKS.iter() {
                    let res = PoaContract::read(*contract_type, *network, *version);
                    if *contract_type == ContractType::Emission && *version == ContractVersion::V1 {
                        assert!(res.is_err());
                    } else {
                        assert!(res.is_ok());
                    }
                }
            }
        }
    }
}
