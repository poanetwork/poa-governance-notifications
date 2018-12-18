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
    XDai,
}

impl Network {
    fn uppercase(&self) -> &str {
        match self {
            Network::Core => "CORE",
            Network::Sokol => "SOKOL",
            Network::XDai=> "XDAI",
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
    fn is_emission(&self) -> bool {
        *self == ContractType::Emission
    }

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
    fn is_v1(&self) -> bool {
        *self == ContractVersion::V1
    }

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
    fn new(
        kind: ContractType,
        version: ContractVersion,
        addr: Address,
        abi: ethabi::Contract,
    ) -> Self {
        PoaContract { kind, version, addr, abi }
    }

    pub fn read(
        contract_type: ContractType,
        network: Network,
        version: ContractVersion,
    ) -> Result<Self> {
        // Exit quickly if we know that the contract does not exist.
        if contract_type.is_emission() && version.is_v1() {
            return Err(Error::EmissionFundsV1ContractDoesNotExist);
        }

        let env_var = format!(
            "{}_CONTRACT_ADDRESS_{}_{:?}",
            contract_type.uppercase(),
            network.uppercase(),
            version
        );
        let contract_addr_str = env::var(&env_var).map_err(|_| Error::MissingEnvVar(env_var))?;
        let contract_addr = Address::from_str(contract_addr_str.trim_left_matches("0x"))
            .map_err(|_| Error::InvalidContractAddr(contract_addr_str.to_string()))?;

        let abi_path = format!(
            "abis/{}/{}",
            version.lowercase(),
            contract_type.abi_file_name()
        );
        let abi_file = File::open(&abi_path).map_err(|_| Error::MissingAbiFile(abi_path.clone()))?;
        let abi = Contract::load(&abi_file).map_err(|_| Error::InvalidAbi(abi_path))?;

        Ok(PoaContract::new(contract_type, version, contract_addr, abi))
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
        if !cli.one_network_specified() {
            return Err(Error::MustSpecifyOneCliArgument("--core, --sokol".to_string()));
        }
        if cli.multiple_versions_specified() {
            return Err(Error::MustSpecifyZeroOrOneCliArguments("--v1, --v2".to_string()));
        }
        if cli.no_contracts_specified() {
            return Err(Error::MustSpecifyAtLeastOneCliArgument(
                "--keys, --threshold, --proxy, --emission".to_string().to_string(),
            ));
        }
        if cli.v1() {
            if cli.xdai() {
                return Err(Error::V1ContractsWereNotDeployedToXDaiChain);
            }
            if cli.emission() {
                return Err(Error::EmissionFundsV1ContractDoesNotExist);
            }
        }
        if !cli.one_start_block_was_specified() {
            return Err(Error::MustSpecifyOneCliArgument(
                "--earliest, --latest, --start-block, --tail".to_string()
            ));
        }

        let network = if cli.core() {
            Network::Core
        } else if cli.sokol() {
            Network::Sokol
        } else {
            Network::XDai
        };

        // We default to `V2` if no contract version CLI argument was supplied.
        let version = if cli.v1() {
            ContractVersion::V1
        } else {
            ContractVersion::V2
        };

        let endpoint_env_var = format!("{}_RPC_ENDPOINT", network.uppercase());
        let endpoint = env::var(&endpoint_env_var)
            .map_err(|_| Error::MissingEnvVar(endpoint_env_var))?;

        let mut contracts = vec![];
        if cli.keys() {
            let keys_contract = PoaContract::read(ContractType::Keys, network, version)?;
            contracts.push(keys_contract);
        }
        if cli.threshold() {
            let threshold_contract = PoaContract::read(ContractType::Threshold, network, version)?;
            contracts.push(threshold_contract);
        }
        if cli.proxy() {
            let proxy_contract = PoaContract::read(ContractType::Proxy, network, version)?;
            contracts.push(proxy_contract);
        }
        if cli.emission() {
            let emission_funds = PoaContract::read(ContractType::Emission, network, version)?;
            contracts.push(emission_funds);
        }

        let start_block = if cli.earliest() {
            StartBlock::Earliest
        } else if cli.latest() {
            StartBlock::Latest
        } else if let Some(start_block_str) = cli.start_block() {
            match start_block_str.parse::<u64>() {
                Ok(block_number) => StartBlock::Number(block_number),
                _ => return Err(Error::InvalidStartBlock(start_block_str.to_string())),
            }
        } else if let Some(tail_str) = cli.tail() {
            match tail_str.parse::<u64>() {
                Ok(tail) => StartBlock::Tail(tail),
                _ => return Err(Error::InvalidTail(tail_str.to_string())),
            }
        } else {
            // TODO: use `DEFAULT_START_BLOCK`?
            unreachable!();
        };

        let block_time = if let Some(n_secs_str) = cli.block_time() {
            n_secs_str.parse().map_err(|_| Error::InvalidBlockTime(n_secs_str.to_string()))?
        } else {
            DEFAULT_BLOCK_TIME_SECS
        };

        let email_notifications = cli.email();

        // TODO: should the recipient email addresses be validated here? For now, we just allow
        // email sending to fail, which will then get logged to the user.
        let email_recipients: Vec<String> = env::var("EMAIL_RECIPIENTS")
            .map_err(|_| Error::MissingEnvVar("EMAIL_RECIPIENTS".to_string()))?
            .split(',')
            .map(|recipient_email_address| recipient_email_address.to_string())
            .collect();

        let smtp_host_domain = if email_notifications {
            match env::var("SMTP_HOST_DOMAIN") {
                Ok(host) => Some(host),
                _ => return Err(Error::MissingEnvVar("SMTP_HOST_DOMAIN".to_string())),
            }
        } else {
            None
        };

        let smtp_port = if email_notifications {
            if let Ok(s) = env::var("SMTP_PORT") {
                let port = s.parse().map_err(|_| Error::InvalidSmtpPort(s.to_string()))?;
                Some(port)
            } else {
                return Err(Error::MissingEnvVar("SMTP_PORT".into()));
            }
        } else {
            None
        };

        let smtp_username = if email_notifications {
            match env::var("SMTP_USERNAME") {
                Ok(username) => Some(username),
                _ => return Err(Error::MissingEnvVar("SMTP_USERNAME".into())),
            }
        } else {
            None
        };

        let smtp_password = if email_notifications {
            match env::var("SMTP_PASSWORD") {
                Ok(password) => Some(password),
                _ => return Err(Error::MissingEnvVar("SMTP_PASSWORD".to_string())),
            }
        } else {
            None
        };

        let outgoing_email_addr = if email_notifications {
            match env::var("OUTGOING_EMAIL_ADDRESS") {
                Ok(outgoing_email_addr) => Some(outgoing_email_addr),
                _ => return Err(Error::MissingEnvVar("OUTGOING_EMAIL_ADDRESS".to_string())),
            }
        } else {
            None
        };

        let notification_limit = if let Some(s) = cli.notification_limit() {
            let limit = s
                .parse()
                .map_err(|_| Error::InvalidNotificationLimit(s.into()))?;
            Some(limit)
        } else {
            None
        };

        let log_emails = cli.log_emails();
        let log_to_file = cli.log_to_file();

        Ok(Config {
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
        })
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::PoaContract;
    use crate::tests::{
        setup, CORE_NETWORK, SOKOL_NETWORK, V1_CONTRACT_TYPES, V1_VERSION, V2_CONTRACT_TYPES,
        V2_VERSION,
    };

    #[test]
    fn test_load_contract_addresses() {
        setup();

        // Load v1 contract addresses.
        for contract_type in V1_CONTRACT_TYPES.iter() {
            let core_env_var = format!(
                "{}_CONTRACT_ADDRESS_CORE_V1",
                contract_type.uppercase(),
            );
            let res = env::var(core_env_var);
            assert!(res.is_ok());

            let sokol_env_var = format!(
                "{}_CONTRACT_ADDRESS_SOKOL_V1",
                contract_type.uppercase(),
            );
            let res = env::var(sokol_env_var);
            assert!(res.is_ok());
        }

        // Load V2 contract addresses.
        for contract_type in V2_CONTRACT_TYPES.iter() {
            let core_env_var = format!(
                "{}_CONTRACT_ADDRESS_CORE_V2",
                contract_type.uppercase(),
            );
            let res = env::var(core_env_var);
            assert!(res.is_ok());

            let sokol_env_var = format!(
                "{}_CONTRACT_ADDRESS_SOKOL_V2",
                contract_type.uppercase(),
            );
            let res = env::var(sokol_env_var);
            assert!(res.is_ok());
        }
    }

    #[test]
    fn test_load_all_contract_abis() {
        setup();

        // Load all of the V1 contracts.
        for contract_type in V1_CONTRACT_TYPES.iter() {
            let res = PoaContract::read(*contract_type, SOKOL_NETWORK, V1_VERSION);
            assert!(res.is_ok());
            let res = PoaContract::read(*contract_type, CORE_NETWORK, V1_VERSION);
            assert!(res.is_ok());
        }

        // Load all of the V2 contracts.
        for contract_type in V2_CONTRACT_TYPES.iter() {
            let res = PoaContract::read(*contract_type, SOKOL_NETWORK, V2_VERSION);
            assert!(res.is_ok());
            let res = PoaContract::read(*contract_type, CORE_NETWORK, V2_VERSION);
            assert!(res.is_ok());
        }
    }
}
