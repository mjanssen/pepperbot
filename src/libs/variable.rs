use std::{env, process};
use log::error;

pub fn get_environment_variable(key: &str) -> String {
    match env::var(key) {
        Ok(val) => val,
        Err(_e) => {
            error!("Missing environment variable: {}", key);
            process::exit(1);
        },
    }
}
