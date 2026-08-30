use std::process::Command;

use hmac_sha512::Hash;
use salvo::prelude::*;

use crate::config::Config;

mod config;
mod hashing;

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

    let mut body: String = "
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
        </html>
    "
    .to_string();

    if let Some(username) = request.form::<&str>("username").await
        && config.user == username
    {
        // Success condition, intentionally left blank.
    } else {
        response.status_code(StatusCode::BAD_REQUEST);
        body = body.replace("{{{CONTENT}}}", "Bad Request");
        response.body(body);
        return;
    }

    if !config.hash_password.is_empty() && !config.hash_salt.is_empty() {
        if let Some(password) = request.form::<&str>("password").await {
            let mut hasher = Hash::new();
            hasher.update(password.as_bytes());

            let mut salt_data = [0u8; 64];
            let result = hex::decode_to_slice(&config.hash_salt, &mut salt_data);
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

            if hash_hexadecimal == config.hash_password {
                // Success condition, intentionally left blank.
            } else {
                response.status_code(StatusCode::BAD_REQUEST);
                body = body.replace("{{{CONTENT}}}", "Bad Request");
                response.body(body);
                return;
            }
        } else {
            response.status_code(StatusCode::BAD_REQUEST);
            body = body.replace("{{{CONTENT}}}", "Bad Request");
            response.body(body);
            return;
        }
    } else {
        if let Some(password) = request.form::<&str>("password").await
            && config.password == password
        {
            // Success condition, intentionally left blank.
        } else {
            response.status_code(StatusCode::BAD_REQUEST);
            body = body.replace("{{{CONTENT}}}", "Bad Request");
            response.body(body);
            return;
        }
    }

    let action_opt: Option<&str> = request.form("action").await;
    if action_opt.is_none() {
        response.status_code(StatusCode::BAD_REQUEST);
        body = body.replace("{{{CONTENT}}}", "Bad Request");
        response.body(body);
        return;
    }

    let action = action_opt.unwrap();

    let mpc_result = do_mpc_command(action, &config).await;

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

    response.render(format!("Accepted\n{}", mpc_result.unwrap()));
}

async fn do_mpc_command(action: &str, config: &Config) -> Result<String, String> {
    let mut output = String::new();
    match action {
        "toggle" => {
            let status = Command::new("mpc")
                .arg(&format!("--host={}", config.mpc_host))
                .arg("toggle")
                .output();
            if let Ok(out) = status {
                output = out
                    .stdout
                    .try_into()
                    .unwrap_or("Unable to be converted to String".to_string());
            } else if let Err(e) = status {
                return Err(format!("Failed to \"toggle\": {}", e.to_string()));
            }
        }
        "next" => {
            let status = Command::new("mpc")
                .arg(&format!("--host={}", config.mpc_host))
                .arg("next")
                .output();
            if let Ok(out) = status {
                output = out
                    .stdout
                    .try_into()
                    .unwrap_or("Unable to be converted to String".to_string());
            } else if let Err(e) = status {
                return Err(format!("Failed to \"next\": {}", e.to_string()));
            }
        }
        "prev" => {
            let status = Command::new("mpc")
                .arg(&format!("--host={}", config.mpc_host))
                .arg("prev")
                .output();
            if let Ok(out) = status {
                output = out
                    .stdout
                    .try_into()
                    .unwrap_or("Unable to be converted to String".to_string());
            } else if let Err(e) = status {
                return Err(format!("Failed to \"prev\": {}", e.to_string()));
            }
        }
        _ => return Err(format!("Invalid action \"{}\"", action)),
    }

    Ok(output)
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

    let router = Router::new()
        .hoop(affix_state::inject(config))
        .get(get_prompt)
        .post(post_prompt);

    Server::new(acceptor).serve(router).await;
}
