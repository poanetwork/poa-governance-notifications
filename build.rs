use std::fs;
use std::path::Path;

/// If no `.env` file exists, this function will create an `.env` file and copy the contents
/// of `sample.env` into it (this is why `sample.env` is kept under source control).
///
///
/// # Panics
///
/// This function panics if both the `.env` file and `sample.env` file do not exist, or if we
/// fail to copy `sample.env` into `.env`.
fn create_env_file_if_dne() {
    let env_path = Path::new(".env");
    if !env_path.exists() {
        let sample_env_path = Path::new("sample.env");
        if !sample_env_path.exists() {
            panic!("neither the `.env` nor the `sample.env` files exist, one of these files must exist to build the `poagov`");
        }
        fs::copy(sample_env_path, env_path)
            .unwrap_or_else(|e| panic!("failed to create the `.env` file from `sample.env`: {:?}", e));
    }
}

fn main() {
    create_env_file_if_dne();
}
