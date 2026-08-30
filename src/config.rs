use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

#[derive(Default, Clone, Debug)]
pub struct Config {
    pub mpc_host: String,
    pub user: String,
    pub password: String,
    pub address_port: String,
}

impl Config {
    pub fn parse_from_config_file(filename: &Path) -> Result<Config, String> {
        let config_content: String =
            fs::read_to_string(filename).map_err(|_| "Failed to open config file!".to_string())?;

        let mut mpc_host_opt: Option<String> = None;
        let mut user_opt: Option<String> = None;
        let mut password_opt: Option<String> = None;
        let mut address_port_opt: Option<String> = None;

        for line in config_content.split('\n') {
            if line.starts_with("mpc_host") {
                let idx = line.find('=').ok_or("Invalid mpc_host line!".to_string())?;
                let mpc_host: String = line[(idx + 1)..].trim().to_string();
                if !mpc_host.is_empty() {
                    mpc_host_opt = Some(mpc_host);
                }
            } else if line.starts_with("user") {
                let idx = line.find('=').ok_or("Invalid mpc_host line!".to_string())?;
                let user: String = line[(idx + 1)..].trim().to_string();
                if !user.is_empty() {
                    user_opt = Some(user);
                }
            } else if line.starts_with("password") {
                let idx = line.find('=').ok_or("Invalid password line!".to_string())?;
                let password: String = line[(idx + 1)..].trim().to_string();
                if !password.is_empty() {
                    password_opt = Some(password);
                }
            } else if line.starts_with("address_port") {
                let idx = line
                    .find('=')
                    .ok_or("Invalid address_port line!".to_string())?;
                let address_port: String = line[(idx + 1)..].trim().to_string();
                if !address_port.is_empty() {
                    address_port_opt = Some(address_port);
                }
            }
        }

        if let Some(mpc_host) = mpc_host_opt
            && let Some(user) = user_opt
            && let Some(password) = password_opt
            && let Some(address_port) = address_port_opt
        {
            Ok(Config {
                mpc_host,
                user,
                password,
                address_port,
            })
        } else {
            Err(String::from("Invalid config file contents!"))
        }
    }

    pub fn parse_config_from_arg() -> Result<Config, String> {
        for arg in std::env::args() {
            if arg.starts_with("--config=") {
                let path = PathBuf::from_str(&arg[9..])
                    .map_err(|_| "Failed to parse arg \"--config=<path>\"!".to_string())?;
                return Self::parse_from_config_file(&path);
            }
        }

        Err(String::from("No valid \"--config=<path>\"!"))
    }
}
