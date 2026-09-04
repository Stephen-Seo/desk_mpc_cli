================================================================================

desk_mpc_cli:

A simple server to do "mpc" commands locally

================================================================================

Invoke via:

./desk_mpc_cli --config=config_file.conf

OR

./desk_mpc_cli --generate


The "--generate" option generates a hash_password and hash_salt with the input
password to be used in the config.


See the "test.config" and "test_hash.config" example config files.


Use nginx as a reverse proxy for hosting with https

    location /mpc {
        rewrite ^/mpc(.*) $1 break;
        proxy_pass http://127.0.0.1:8080;
    }
