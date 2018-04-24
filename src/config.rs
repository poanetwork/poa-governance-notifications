use std::env;
use std::fmt::{self, Debug, Display, Formatter};
use std::fs::File;
use std::str::FromStr;
use std::time::Duration;

use dotenv::dotenv;
use ethabi::{Contract, Event, Function};
use ethereum_types::Address;
use web3::types::BlockNumber;

use cli::Cli;
use utils::hex_string_to_u64;

#[derive(Clone, Copy, Debug)]
pub enum Network { Core, Sokol, Local }

impl<'a> From<&'a str> for Network {
    fn from(s: &'a str) -> Self {
        match s {
            "core" => Network::Core,
            "sokol" => Network::Sokol,
            "local" => Network::Local,
            _ => panic!(format!("Invalid network: {}", s))
        }
    }
}

impl Display for Network {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Network::Core => write!(f, "core"),
            Network::Sokol => write!(f, "sokol"),
            Network::Local => write!(f, "local")
        }
    }
}

impl Network {
    fn to_uppercase(&self) -> String {
        format!("{}", self).to_uppercase()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ContractType { Keys, Threshold, Proxy }

impl<'a> From<&'a str> for ContractType {
    fn from(s: &'a str) -> Self {
        match s {
            "keys" => ContractType::Keys,
            "threshold" => ContractType::Threshold,
            "proxy" => ContractType::Proxy,
            _ => panic!("Invalid contract type: {}", s)
        }
    }
}

impl Display for ContractType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            ContractType::Keys => write!(f, "keys"),
            ContractType::Threshold => write!(f, "threshold"),
            ContractType::Proxy => write!(f, "proxy")
        }
    }
}

impl ContractType {
    fn to_uppercase(&self) -> String {
        format!("{}", self).to_uppercase()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StartBlock {
    pub block: BlockNumber,
    pub tail: u64
}

impl StartBlock {
    fn latest() -> Self {
        StartBlock { block: BlockNumber::Latest, tail: 0 }
    }

    fn earliest() -> Self {
        StartBlock { block: BlockNumber::Earliest, tail: 0 }
    }

    fn tail(tail: u64) -> Self {
        StartBlock { block: BlockNumber::Latest, tail }
    }
}

impl<'a> From<&'a str> for StartBlock {
    fn from(s: &'a str) -> Self {
        let mut start_block = StartBlock::latest();

        if s.starts_with("-") {
            start_block.tail = s[1..].parse().expect("Invalid start-block");
        } else if s == "earliest" {
            start_block.block = BlockNumber::Number(0);
        } else if s.starts_with("0x") {
            start_block.block = hex_string_to_u64(s).expect("Invalid start-block").into();
        } else if s != "latest" {
            start_block.block = s.parse::<u64>().expect("Invalid start-block").into();
        }

        start_block
    }
}

#[derive(Clone, Debug)]
pub struct Validator {
    pub name: String,
    pub email: String
}

pub struct PoaContract {
    pub kind: ContractType,
    pub addr: Address,
    pub abi: Contract
}

impl PoaContract {
    fn new(kind: ContractType, addr: Address, abi: Contract) -> Self {
        PoaContract { kind, addr, abi }
    }

    pub fn event(&self, event: &str) -> Event {
        self.abi.event(event).unwrap().clone()
    }

    pub fn function(&self, function: &str) -> Function {
        self.abi.function(function).unwrap().clone()
    }
}

impl Display for PoaContract {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "PoaContract({:?}, {:?})", self.kind, self.addr)
    }
}

impl Debug for PoaContract {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Debug)]
pub struct Config {
    pub network: Network,
    pub endpoint: String,
    pub contracts: Vec<PoaContract>,
    pub start_block: StartBlock,
    pub send_email_notifications: bool,
    pub send_push_notifications: bool,
    pub validators: Vec<Validator>,
    pub avg_block_time: Duration,
    pub smtp_host_domain: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub outgoing_email: String
}

impl Config {
    pub fn load() -> Self {
        dotenv().ok();
        let cli = Cli::load();

        let network = if let Some(s) = cli.value_of("network") {
            s.into()
        } else if cli.is_present("core") {
            Network::Core
        } else if cli.is_present("sokol") {
            Network::Sokol
        } else if cli.is_present("local") {
            Network::Local
        } else {
            env::var("USE_NETWORK").unwrap().as_str().into()
        };

        let network_uppercase = network.to_uppercase();

        let endpoint = if let Some(s) = cli.value_of("rpc") {
            s.into()
        } else {
            let env_var = format!("{}_RPC_ENDPOINT", network_uppercase);
            env::var(&env_var).unwrap()
        };

        let mut contract_types: Vec<ContractType> = vec![];
        if let Some(s) = cli.value_of("monitor") {
            s.split(',').for_each(|s| contract_types.push(s.into()));
        }
        if cli.is_present("keys") {
            contract_types.push(ContractType::Keys);
        }
        if cli.is_present("threshold") {
            contract_types.push(ContractType::Threshold);
        }
        if cli.is_present("proxy") {
            contract_types.push(ContractType::Proxy);
        }
        if contract_types.is_empty() {
            env::var("MONITOR_BALLOTS").unwrap().split(',')
                .for_each(|s| contract_types.push(s.into()));
        }

        let contracts: Vec<PoaContract> = contract_types.iter()
            .map(|contract_type| {
                let env_var = format!(
                    "{}_{}_CONTRACT_ADDRESS",
                    network_uppercase,
                    contract_type.to_uppercase()
                );
                let hex = env::var(&env_var)
                    .expect(&format!("Contract address not found: {}", env_var));
                let addr = Address::from_str(hex.trim_left_matches("0x")).unwrap();
                let abi_path = format!("abis/{}/{}.json", network, contract_type);
                let file = File::open(&abi_path)
                    .expect(&format!("ABI file not found: {}", abi_path));
                let abi = Contract::load(&file)
                    .expect(&format!("Invalid ABI file: {}", abi_path));
                PoaContract::new(*contract_type, addr, abi)
            })
            .collect();

        let start_block = if let Some(s) = cli.value_of("start_block") {
            s.into()
        } else if cli.is_present("earliest") {
            StartBlock::earliest()
        } else if cli.is_present("latest") {
            StartBlock::latest()
        } else if let Some(s) = cli.value_of("tail") {
            StartBlock::tail(s.parse().expect("Invalid tail value"))
        } else {
            env::var("START_BLOCK").unwrap().as_str().into()
        };

        let send_email_notifications = if cli.is_present("email") {
            true
        } else {
            env::var("SEND_EMAIL_NOTIFICATIONS").unwrap().parse().unwrap()
        };

        let send_push_notifications = if cli.is_present("push") {
            true
        } else {
            env::var("SEND_PUSH_NOTIFICATIONS").unwrap().parse().unwrap()
        };

        let validators = env::var("VALIDATORS").unwrap().split(',')
            .map(|s| Validator { email: s.into(), name: "".into() })
            .collect();

        let avg_block_time = if let Some(s) = cli.value_of("block_time") {
            Duration::from_secs(s.parse().unwrap())
        } else {
            let s = env::var("AVG_BLOCK_TIME_SECS").unwrap();
            Duration::from_secs(s.parse().unwrap())
        };

        let smtp_host_domain = env::var("SMTP_HOST_DOMAIN").unwrap();
        let smtp_port = env::var("SMTP_PORT").unwrap().parse().unwrap();
        let smtp_username = env::var("SMTP_USERNAME").unwrap();
        let smtp_password = env::var("SMTP_PASSWORD").unwrap();
        let outgoing_email = env::var("OUTGOING_EMAIL_ADDRESS").unwrap();

        Config {
            network, endpoint, contracts, start_block,
            send_email_notifications, send_push_notifications,
            validators, avg_block_time, smtp_host_domain,
            smtp_port, smtp_username, smtp_password, outgoing_email
        }
    }
}
