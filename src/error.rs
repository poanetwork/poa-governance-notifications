use ctrlc;
use dotenv;
use jsonrpc_core;
use ethabi;
use failure;
use lettre;
use native_tls;
use reqwest;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    CtrlcError(ctrlc::Error),
    EmissionFundsV1ContractDoesNotExist,
    EnvFileNotFound(dotenv::Error),
    FailedToBuildEmail(failure::Error),
    FailedToBuildRequest(reqwest::Error),
    FailedToBuildTls(native_tls::Error),
    FailedToParseBallotCreatedLog(String),
    FailedToParseEnvFile(dotenv::Error),
    FailedToParseRawLogToLog(ethabi::Error),
    FailedToResolveSmtpHostDomain(lettre::smtp::error::Error),
    FailedToSendEmail(lettre::smtp::error::Error),
    InvalidAbi(String),
    InvalidBlockTime(String),
    InvalidContractAddr(String),
    InvalidNotificationLimit(String),
    InvalidSmtpPort(String),
    InvalidStartBlock(String),
    InvalidTail(String),
    JsonRpcResponseFailure(jsonrpc_core::types::response::Failure),
    MissingAbiFile(String),
    MissingEnvVar(String),
    MustSpecifyAtLeastOneCliArgument(String),
    MustSpecifyOneCliArgument(String),
    RequestFailed(reqwest::Error),
    StartBlockExceedsLastBlockMined {
        start_block: u64,
        last_mined_block: u64,
    },
}
