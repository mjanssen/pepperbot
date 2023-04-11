use log::{info};
use std::{env, fs};

pub fn get_app_version() -> String {
    let filepath = "/etc/pepperbot_build";
    match fs::read_to_string(filepath) {
        Ok(val) => val,
        Err(_e) => "__NO_VERSION_AVAILABLE__".to_string()
    }
}

pub fn get_helm_chart_version() -> String {
    let key = "HELM_CHART_VERSION";
    match env::var(key) {
        Ok(val) => val,
        Err(_e) => "__NO_VERSION_AVAILABLE__".to_string()
    }
}

pub fn print_version() -> () {
    let app_version = get_app_version();
    let helm_chart_version = get_helm_chart_version();

    info!("App: {app_version}");
    info!("Helm Chart: {helm_chart_version}");
}
