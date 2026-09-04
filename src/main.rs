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
    process::Command,
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};

use hmac_sha512::Hash;
use salvo::prelude::*;
use tokio::time::{Instant, sleep_until};

use crate::config::Config;

mod config;
mod hashing;

const CACHE_LIFETIME: Duration = Duration::from_secs(120);

#[derive(Debug, Clone)]
struct CacheStruct {
    mpc_output: String,
    created_instant: Instant,
}

impl CacheStruct {
    pub fn new(output: String) -> Self {
        Self {
            mpc_output: output,
            created_instant: Instant::now(),
        }
    }
}

type OutputCacheT = Arc<Mutex<HashMap<String, CacheStruct>>>;

const COMMON_BODY: &str = "
        <html>
        <head>
            <style>
            body {
                color: #FFF;
                background-color: #333;
            }
            </style>
        </head>
        <body>
        {{{CONTENT}}}
        </body>
        </html>";

#[handler]
async fn get_prompt(response: &mut Response) {
    response.body(
        "
            <html>
            <head>

            <style>
            body {
                color: #FFF;
                background-color: #333;
            }
            </style>

            </head>
            <body>
            <form action=\"\" method=\"post\" class=\"prompt_form\">

            <label for=\"username\">Username:</label>
            <input type=\"text\" name=\"username\" id=\"username\" required />

            <br />

            <label for\"password\">Password:</label>
            <input type=\"password\" name=\"password\" id=\"password\" required />

            <br />

            <fieldset>
            <legend>Action</legend>
            <label><input type=\"radio\" name=\"action\" value=\"toggle\" /> Toggle</label>
            <label><input type=\"radio\" name=\"action\" value=\"next\" /> Next</label>
            <label><input type=\"radio\" name=\"action\" value=\"prev\" /> Prev</label>
            <label><input type=\"radio\" name=\"action\" value=\"status\" /> Status</label>
            <label><input type=\"radio\" name=\"action\" value=\"single_mode\" /> Single Mode</label>
            </fieldset>

            <br />
            <br />

            <input type=\"submit\" value=\"Submit\" />
            
            </form>
            </body>
            </html>
        ",
    );
}

#[handler]
async fn post_prompt(request: &mut Request, depot: &Depot, response: &mut Response) {
    let config = depot.get_typed::<Config>().unwrap();
    let output_cache: &OutputCacheT = depot.get_typed().unwrap();

    let mut body: String = COMMON_BODY.to_string();

    let sleep_instant: Instant = Instant::now() + Duration::from_millis(1500);

    let user_pass: &config::UserPass;

    if let Some(username) = request.form::<&str>("username").await
        && let Some(config_user_pass) = config.users.get(username)
    {
        // Success condition
        user_pass = config_user_pass;
    } else {
        sleep_until(sleep_instant).await;
        response.status_code(StatusCode::BAD_REQUEST);
        body = body.replace("{{{CONTENT}}}", "Bad Request");
        response.body(body);
        return;
    }

    if !user_pass.hash_password.is_empty() && !user_pass.hash_salt.is_empty() {
        if let Some(password) = request.form::<&str>("password").await {
            let mut hasher = Hash::new();
            hasher.update(password.as_bytes());

            let mut salt_data = [0u8; 64];
            let result = hex::decode_to_slice(&user_pass.hash_salt, &mut salt_data);
            if let Err(e) = result {
                eprintln!("ERROR: Failed to decode hexadecimal salt: {}", e);
                response.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                body = body.replace("{{{CONTENT}}}", "Internal Server Error");
                response.body(body);
                return;
            }

            hasher.update(salt_data);
            let hash = hasher.finalize();

            let hash_hexadecimal = hex::encode(hash);

            if hash_hexadecimal == user_pass.hash_password {
                // Success condition, intentionally left blank
            } else {
                sleep_until(sleep_instant).await;
                response.status_code(StatusCode::BAD_REQUEST);
                body = body.replace("{{{CONTENT}}}", "Bad Request");
                response.body(body);
                return;
            }
        } else {
            sleep_until(sleep_instant).await;
            response.status_code(StatusCode::BAD_REQUEST);
            body = body.replace("{{{CONTENT}}}", "Bad Request");
            response.body(body);
            return;
        }
    } else {
        if let Some(password) = request.form::<&str>("password").await
            && user_pass.password == password
        {
            // Success condition, intentionally left blank
        } else {
            sleep_until(sleep_instant).await;
            response.status_code(StatusCode::BAD_REQUEST);
            body = body.replace("{{{CONTENT}}}", "Bad Request");
            response.body(body);
            return;
        }
    }

    let action_opt: Option<&str> = request.form("action").await;
    if action_opt.is_none() {
        sleep_until(sleep_instant).await;
        response.status_code(StatusCode::BAD_REQUEST);
        body = body.replace("{{{CONTENT}}}", "Bad Request");
        response.body(body);
        return;
    }

    sleep_until(sleep_instant).await;

    let action = action_opt.unwrap();

    let mpc_result = do_mpc_command(action, config).await;

    if let Err(e) = mpc_result {
        response.status_code(StatusCode::INTERNAL_SERVER_ERROR);
        eprintln!(
            "ERROR: Failed to invoke mpc (action {}) error: {}",
            action, e
        );
        body = body.replace("{{{CONTENT}}}", "Internal Server Error");
        response.body(body);
        return;
    }

    let mut random_slice = [0u8; 64];
    let random_result = getrandom::fill(&mut random_slice)
        .map_err(|e| format!("ERROR: Failed to get random data: {}", e));

    if let Err(e) = random_result {
        response.status_code(StatusCode::INTERNAL_SERVER_ERROR);
        eprintln!("ERROR: Failed to fill slice with random data: {}", e);
        body = body.replace("{{{CONTENT}}}", "Internal Server Error");
        response.body(body);
        return;
    }

    let random_key = hex::encode(random_slice);

    {
        let mut map = output_cache
            .lock()
            .expect("Should be able to get lock on output_map");

        map.insert(
            random_key.to_owned(),
            CacheStruct::new(
                mpc_result
                    .expect("Should be output from mpc")
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('\n', "<br />"),
            ),
        );
    }

    response.render(Redirect::found(format!("result/{}", random_key)));
}

async fn do_mpc_command(action: &str, config: &Config) -> Result<String, String> {
    let mut output = String::new();
    match action {
        "toggle" => {
            let status = Command::new("mpc")
                .arg(format!("--host={}", config.mpc_host))
                .arg("toggle")
                .output();
            if let Ok(out) = status {
                output = out
                    .stdout
                    .try_into()
                    .unwrap_or("Unable to be converted to String".to_string());
            } else if let Err(e) = status {
                return Err(format!("Failed to \"toggle\": {}", e));
            }
        }
        "next" => {
            let status = Command::new("mpc")
                .arg(format!("--host={}", config.mpc_host))
                .arg("next")
                .output();
            if let Ok(out) = status {
                output = out
                    .stdout
                    .try_into()
                    .unwrap_or("Unable to be converted to String".to_string());
            } else if let Err(e) = status {
                return Err(format!("Failed to \"next\": {}", e));
            }
        }
        "prev" => {
            let status = Command::new("mpc")
                .arg(format!("--host={}", config.mpc_host))
                .arg("prev")
                .output();
            if let Ok(out) = status {
                output = out
                    .stdout
                    .try_into()
                    .unwrap_or("Unable to be converted to String".to_string());
            } else if let Err(e) = status {
                return Err(format!("Failed to \"prev\": {}", e));
            }
        }
        "status" => {
            let status = Command::new("mpc")
                .arg(format!("--host={}", config.mpc_host))
                .output();
            if let Ok(out) = status {
                output = out
                    .stdout
                    .try_into()
                    .unwrap_or("Unable to be converted to String".to_string());
            } else if let Err(e) = status {
                return Err(format!("Failed to \"status\": {}", e));
            }
        }
        "single_mode" => {
            let status = Command::new("mpc")
                .arg(format!("--host={}", config.mpc_host))
                .arg("single")
                .output();
            if let Ok(out) = status {
                output = out
                    .stdout
                    .try_into()
                    .unwrap_or("Unable to be converted to String".to_string());
            } else if let Err(e) = status {
                return Err(format!("Failed to \"single\": {}", e));
            }
        }
        _ => return Err(format!("Invalid action \"{}\"", action)),
    }

    Ok(output)
}

#[handler]
async fn get_cached_output(response: &mut Response, request: &mut Request, depot: &Depot) {
    let mut body = COMMON_BODY.to_string();
    let id_opt = request.param::<String>("id");
    if id_opt.is_none() {
        response.status_code(StatusCode::BAD_REQUEST);
        body = body.replace("{{{CONTENT}}}", "Bad Request");
        response.body(body);
        return;
    }

    let output_cache: &OutputCacheT = depot.get_typed().unwrap();

    let mut cache_lock = output_cache
        .lock()
        .expect("Should be able to lock output cache Mutex");

    let id = id_opt.unwrap();

    if let Some(v) = cache_lock.get(&id) {
        body = body.replace("{{{CONTENT}}}", &v.mpc_output);
        response.body(body);
        cache_lock.remove(&id);
    } else {
        response.status_code(StatusCode::BAD_REQUEST);
        body = body.replace("{{{CONTENT}}}", "Bad Request");
        response.body(body);
    }
}

#[tokio::main]
async fn main() {
    for arg in std::env::args() {
        if arg == "--generate" {
            hashing::interactive_gen_hash_salt()
                .expect("Should be able to generate hash and salt!");
            return;
        }
    }

    let config: Config = Config::parse_config_from_arg()
        .expect("Config file should be specified via \"--config=<filename>\"!");

    let acceptor = TcpListener::new(config.address_port.clone()).bind().await;

    eprintln!("Listening on: {}", config.address_port);

    let mpc_output_cache: OutputCacheT = Arc::new(Mutex::new(HashMap::new()));

    let mpc_output_cache_clone = mpc_output_cache.clone();

    thread::spawn(move || {
        loop {
            sleep(CACHE_LIFETIME);

            let mut lock = mpc_output_cache_clone
                .lock()
                .expect("Should be able to unlock Mutex");
            let mut to_remove = Vec::new();

            for (k, v) in lock.iter() {
                if v.created_instant.elapsed() > CACHE_LIFETIME {
                    to_remove.push(k.clone());
                }
            }

            for key in to_remove {
                lock.remove(&key);
            }
        }
    });

    let router = Router::new()
        .hoop(affix_state::inject(config))
        .hoop(affix_state::inject(mpc_output_cache))
        .get(get_prompt)
        .post(post_prompt)
        .push(Router::with_path("result/{id}").get(get_cached_output));

    Server::new(acceptor).serve(router).await;
}
