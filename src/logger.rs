use std::fs::{self, create_dir, File, read_dir, remove_file};
use std::io::stderr;
use std::path::Path;

use chrono::{DateTime, TimeZone, Utc};
use slog::{self, Drain};
use slog_term::{FullFormat, PlainSyncDecorator};
use web3::types::BlockNumber;

use config::Config;
use error::Error;
use notify::Notification;

// The date format used to name log files; e.g. "Oct-08-2018-14:09:00".
const FILE_NAME_DATE_FORMAT: &str = "%b-%d-%Y-%H:%M:%S";
// The directory (relative to Cargo.toml) to store logs.
const LOGS_DIR: &str = "logs";
const MAX_NUMBER_OF_LOG_FILES: usize = 3;
const MAX_LOG_FILE_SIZE_MB: usize = 4;
const MAX_LOG_FILE_SIZE_BYTES: usize = MAX_LOG_FILE_SIZE_MB * 1024 * 1024;
// We dont want to check the log file's size after every log that is written, this constant states
// "after this many logs have been written, check the log file's size". This value assumes an
// average log is around 100 ASCII characters (bytes) long. 
const INITIAL_CHECK_FILE_SIZE_AT: usize = MAX_LOG_FILE_SIZE_BYTES / 100;

fn create_logs_dir() {
    let logs_dir = Path::new(LOGS_DIR);
    if !logs_dir.exists() {
        create_dir(logs_dir)
            .unwrap_or_else(|e| panic!("could not create ./logs directory: {:?}", e));
    }
}

fn read_logs_dir() -> Vec<LogFile> {
    let mut log_files: Vec<LogFile> = read_dir(LOGS_DIR)
        .unwrap_or_else(|e| panic!("could not read ./logs directory: {:?}", e))
        .filter_map(|res| {
            let path = res.ok()?.path();
            let file_name = path.file_name().unwrap().to_str().unwrap();
            LogFile::from_file_name(file_name).ok()
        }).collect();
    log_files.sort_unstable();
    log_files
}

fn rotate_log_files(log_files: &mut Vec<LogFile>) -> File {
    while log_files.len() >= MAX_NUMBER_OF_LOG_FILES {
        let log_file_to_remove = log_files.remove(0);
        log_file_to_remove.remove_file();
    }
    let log_file = LogFile::now();
    let file = log_file.create_file();
    log_files.push(log_file);
    file
}

fn get_file_size_in_bytes(path: &str) -> usize {
    fs::metadata(&path)
        .unwrap_or_else(|_| panic!("log file does not exist: {}", path))
        .len() as usize
}

enum LogLocation {
    Stderr,
    File(File),
}

fn create_slog_logger(log_location: LogLocation) -> slog::Logger {
    if let LogLocation::File(file) = log_location {
        let decorator = PlainSyncDecorator::new(file);
        let drain = FullFormat::new(decorator).build().fuse();
        slog::Logger::root(drain, o!())
    } else {
        let decorator = PlainSyncDecorator::new(stderr());
        let drain = FullFormat::new(decorator).build().fuse();
        slog::Logger::root(drain, o!())
    }
}

#[derive(Eq, Ord, PartialEq, PartialOrd)]
struct LogFile(DateTime<Utc>);

impl LogFile {
    fn now() -> Self {
        LogFile(Utc::now())
    }
    
    fn from_file_name(file_name: &str) -> Result<Self, ()> {
        if let Ok(dt) = Utc.datetime_from_str(file_name, FILE_NAME_DATE_FORMAT) {
            Ok(LogFile(dt))
        } else {
            Err(())
        }
    }

    fn file_name(&self) -> String {
        self.0.format(FILE_NAME_DATE_FORMAT).to_string()
    }

    fn path(&self) -> String {
        format!("{}/{}", LOGS_DIR, self.file_name())
    }

    fn create_file(&self) -> File {
        let path = self.path();
        File::create(&path)
            .unwrap_or_else(|_| panic!("failed to create log file: {}", path))
    }

    fn remove_file(&self) {
        let path = self.path();
        remove_file(&path)
            .unwrap_or_else(|_| panic!("failed to delete log file: {}", path))
    }
}

pub struct Logger {
    logger: slog::Logger,
    log_files: Vec<LogFile>,
    log_count: usize,
    check_file_size_at: usize,
}

impl Logger {
    pub fn new(config: &Config) -> Self {
        let (logger, log_files) = if config.log_to_file {
            create_logs_dir();
            let mut log_files = read_logs_dir();
            let current_log_file = rotate_log_files(&mut log_files);
            let logger = create_slog_logger(LogLocation::File(current_log_file));
            (logger, log_files)
        } else {
            let logger = create_slog_logger(LogLocation::Stderr);
            (logger, vec![])
        };
        Logger {
            logger,
            log_files,
            log_count: 0,
            check_file_size_at: INITIAL_CHECK_FILE_SIZE_AT,
        }
    }
    
    fn logging_to_file(&self) -> bool {
        !self.log_files.is_empty()
    }

    fn should_rotate_log_file(&mut self) -> bool {
        if self.logging_to_file() {
            if self.log_count >= self.check_file_size_at {
                let path = self.log_files.last().unwrap().path();
                let file_size = get_file_size_in_bytes(&path);
                if file_size >= MAX_LOG_FILE_SIZE_BYTES {
                    return true;
                } 
                let avg_bytes_per_log = file_size / self.log_count;
                let remaining_bytes = MAX_LOG_FILE_SIZE_BYTES - file_size;
                let remaining_logs = remaining_bytes / avg_bytes_per_log;
                self.check_file_size_at += remaining_logs;
            }
        }
        false
    }

    fn rotate_log_file(&mut self) {
        let new_log_file = rotate_log_files(&mut self.log_files);
        self.logger = create_slog_logger(LogLocation::File(new_log_file));
        self.log_count = 0;
        self.check_file_size_at = INITIAL_CHECK_FILE_SIZE_AT;
    }

    fn increment_log_count(&mut self) {
        self.log_count += 1;
        if self.should_rotate_log_file() {
            self.rotate_log_file();
        }
    }

    pub fn log_starting_poagov(&mut self) {
        info!(&self.logger, "starting poagov...");
        self.increment_log_count();
    }
    
    pub fn log_ctrlc(&mut self) {
        warn!(&self.logger, "recieved ctrl-c signal, gracefully shutting down...");
        self.increment_log_count();
    }

    pub fn log_no_email_recipients_configured(&mut self) {
        warn!(&self.logger, "email notifications are enabled, but there are no email recipients");
        self.increment_log_count();
    }

    pub fn log_notification_email_body(&mut self, notif: &Notification) {
        info!(&self.logger, "governance notification\n{}", notif.email_text());
        self.increment_log_count();
    }
    
    pub fn log_notification(&mut self, notif: &Notification) {
        let ballot_created_log = notif.log();
        info!(
            &self.logger,
            "governance notification";
            "ballot" => format!("{:?}", ballot_created_log.ballot_type),
            "ballot_id" => format!("{}", ballot_created_log.ballot_id),
            "block_number" => format!("{}", ballot_created_log.block_number)
        );
        self.increment_log_count();
    }
    
    pub fn log_failed_to_build_email(&mut self, e: Error) {
        warn!(&self.logger, "failed to build email"; "error" => format!("{:?}", e));
        self.increment_log_count();
    }
    
    pub fn log_failed_to_send_email(&mut self, recipient: &str, e: Error) {
        warn!(
            &self.logger,
            "failed to send email";
            "recipient" => recipient,
            "error" => format!("{:?}", e)
        );
        self.increment_log_count();
    }

    pub fn log_email_sent(&mut self, recipient: &str) {
        info!(&self.logger, "email sent"; "to" => recipient);
        self.increment_log_count();
    }
    
    pub fn log_reached_notification_limit(&mut self, notification_limit: usize) {
        warn!(
            &self.logger,
            "reached notification limit, gracefully shutting down...";
            "limit" => notification_limit
        );
        self.increment_log_count();
    }

    pub fn log_finished_block_window(&mut self, start: BlockNumber, stop: BlockNumber) {
        let block_range = format!("{:?}...{:?}", start, stop);
        info!(&self.logger, "finished checking blocks"; "block_range" => block_range);
        self.increment_log_count();
    }
}
