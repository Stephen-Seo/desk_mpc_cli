// ISC License
//
// Copyright (c) 2026 Stephen Seo
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
// REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
// AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
// INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
// LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
// OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
// PERFORMANCE OF THIS SOFTWARE.

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    str::FromStr,
};

// UserPass contains either a "password" and no "hash_*" entries, or both
// "hash_*" entries and no "password".
#[derive(Default, Clone, Debug)]
pub struct UserPass {
    pub password: String,
    pub hash_password: String,
    pub hash_salt: String,
}

#[derive(Default, Clone, Debug)]
pub struct Config {
    pub mpc_host: String,
    // username is the key.
    pub users: HashMap<String, UserPass>,
    pub address_port: String,
}

impl Config {
    pub fn parse_from_config_file(filename: &Path) -> Result<Config, String> {
        let reader = BufReader::new(
            File::open(filename).map_err(|_| "Failed to open config file!".to_string())?,
        );

        let mut mpc_host_opt: Option<String> = None;
        let mut user_opt: Option<String> = None;
        let mut password_opt: Option<String> = None;
        let mut hash_password_opt: Option<String> = None;
        let mut hash_salt_opt: Option<String> = None;
        let mut address_port_opt: Option<String> = None;

        let mut users: HashMap<String, UserPass> = HashMap::new();
        for line in reader.lines() {
            let line = line.map_err(|_| "Failed to read line from config file!".to_string())?;
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

                if let Some(ref user) = user_opt {
                    if let Some(password) = password_opt {
                        users.insert(
                            user.to_owned(),
                            UserPass {
                                password,
                                hash_password: String::new(),
                                hash_salt: String::new(),
                            },
                        );

                        user_opt = None;
                        password_opt = None;

                        hash_password_opt = None;
                        hash_salt_opt = None;
                    } else if let Some(ref hash_password) = hash_password_opt
                        && let Some(hash_salt) = hash_salt_opt
                    {
                        if hex::decode(hash_password).is_err() {
                            return Err(String::from(
                                "Invalid \"hash_password\" (not hexadecimal)!",
                            ));
                        }
                        if hex::decode(&hash_salt).is_err() {
                            return Err(String::from("Invalid \"hash_salt\" (not hexadecimal)!"));
                        }

                        users.insert(
                            user.to_owned(),
                            UserPass {
                                password: String::new(),
                                hash_password: hash_password.to_owned(),
                                hash_salt,
                            },
                        );

                        user_opt = None;
                        password_opt = None;

                        hash_password_opt = None;
                        hash_salt_opt = None;
                    }
                }
            } else if line.starts_with("password") {
                let idx = line.find('=').ok_or("Invalid password line!".to_string())?;
                let password: String = line[(idx + 1)..].trim().to_string();
                if !password.is_empty() {
                    password_opt = Some(password);
                }

                if let Some(ref user) = user_opt
                    && let Some(password) = password_opt
                {
                    users.insert(
                        user.to_owned(),
                        UserPass {
                            password,
                            hash_password: String::new(),
                            hash_salt: String::new(),
                        },
                    );

                    user_opt = None;
                    password_opt = None;

                    hash_password_opt = None;
                    hash_salt_opt = None;
                }
            } else if line.starts_with("hash_password") {
                let idx = line
                    .find('=')
                    .ok_or("Invalid hash_password line!".to_string())?;
                let hash_password: String = line[(idx + 1)..].trim().to_string();
                if !hash_password.is_empty() {
                    hash_password_opt = Some(hash_password);
                }

                if let Some(ref user) = user_opt
                    && let Some(ref hash_password) = hash_password_opt
                    && let Some(hash_salt) = hash_salt_opt
                {
                    if hex::decode(hash_password).is_err() {
                        return Err(String::from("Invalid \"hash_password\" (not hexadecimal)!"));
                    }
                    if hex::decode(&hash_salt).is_err() {
                        return Err(String::from("Invalid \"hash_salt\" (not hexadecimal)!"));
                    }

                    users.insert(
                        user.to_owned(),
                        UserPass {
                            password: String::new(),
                            hash_password: hash_password.to_owned(),
                            hash_salt,
                        },
                    );

                    user_opt = None;
                    password_opt = None;

                    hash_password_opt = None;
                    hash_salt_opt = None;
                }
            } else if line.starts_with("hash_salt") {
                let idx = line
                    .find('=')
                    .ok_or("Invalid hash_salt line!".to_string())?;
                let hash_salt: String = line[(idx + 1)..].trim().to_string();
                if !hash_salt.is_empty() {
                    hash_salt_opt = Some(hash_salt);
                }

                if let Some(ref user) = user_opt
                    && let Some(ref hash_password) = hash_password_opt
                    && let Some(hash_salt) = hash_salt_opt
                {
                    if hex::decode(hash_password).is_err() {
                        return Err(String::from("Invalid \"hash_password\" (not hexadecimal)!"));
                    }
                    if hex::decode(&hash_salt).is_err() {
                        return Err(String::from("Invalid \"hash_salt\" (not hexadecimal)!"));
                    }

                    users.insert(
                        user.to_owned(),
                        UserPass {
                            password: String::new(),
                            hash_password: hash_password.to_owned(),
                            hash_salt,
                        },
                    );

                    user_opt = None;
                    password_opt = None;

                    hash_password_opt = None;
                    hash_salt_opt = None;
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
            && let Some(address_port) = address_port_opt
            && !users.is_empty()
        {
            Ok(Config {
                mpc_host,
                users,
                address_port,
            })
        } else {
            Err(String::from("Invalid config file contents!"))
        }
    }

    pub fn parse_config_from_arg() -> Result<Config, String> {
        for arg in std::env::args() {
            if let Some(arg) = arg.strip_prefix("--config=") {
                let path = PathBuf::from_str(arg)
                    .map_err(|_| "Failed to parse arg \"--config=<path>\"!".to_string())?;
                return Self::parse_from_config_file(&path);
            }
        }

        Err(String::from("No valid \"--config=<path>\"!"))
    }
}
