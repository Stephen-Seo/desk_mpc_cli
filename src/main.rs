use std::process::Command;

use salvo::prelude::*;

use crate::config::Config;

mod config;

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

    body = body.replace("{{{CONTENT}}}", "Accepted");
    response.body(body);
}

async fn do_mpc_command(action: &str, config: &Config) -> Result<(), String> {
    match action {
        "toggle" => {
            let status = Command::new("mpc")
                .arg(&format!("--host={}", config.mpc_host))
                .arg("toggle")
                .output();
            if let Err(e) = status {
                return Err(format!("Failed to \"toggle\": {}", e.to_string()));
            }
        }
        "next" => {
            let status = Command::new("mpc")
                .arg(&format!("--host={}", config.mpc_host))
                .arg("next")
                .output();
            if let Err(e) = status {
                return Err(format!("Failed to \"next\": {}", e.to_string()));
            }
        }
        "prev" => {
            let status = Command::new("mpc")
                .arg(&format!("--host={}", config.mpc_host))
                .arg("prev")
                .output();
            if let Err(e) = status {
                return Err(format!("Failed to \"prev\": {}", e.to_string()));
            }
        }
        _ => return Err(format!("Invalid action \"{}\"", action)),
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let config: Config = Config::parse_config_from_arg()
        .expect("Config file should be specified via \"--config=<filename>\"!");

    let acceptor = TcpListener::new(config.address_port.clone()).bind().await;

    let router = Router::new()
        .hoop(affix_state::inject(config))
        .get(get_prompt)
        .post(post_prompt);

    Server::new(acceptor).serve(router).await;
}
