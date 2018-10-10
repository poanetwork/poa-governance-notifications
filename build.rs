use std::fs;
use std::path::Path;

fn create_env_file_if_dne() {
    let env_path = Path::new(".env");
    if !env_path.exists() {
        let sample_env_path = Path::new("sample.env");
        if !sample_env_path.exists() {
            panic!("neither the .env nor the sample.env files exist");
        }
        fs::copy(sample_env_path, env_path)
            .unwrap_or_else(|e| panic!("could not create .env file from sample.env: {:?}", e));
    }
}

fn main() {
    create_env_file_if_dne();
}
