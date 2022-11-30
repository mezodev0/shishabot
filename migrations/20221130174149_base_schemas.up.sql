CREATE TABLE blacklisted_servers (
    server_id INT8,
    reason    VARCHAR(256),
    PRIMARY KEY (server_id)
)

CREATE TABLE blacklisted_users (
    user_id INT8,
    reason  VARCHAR(256),
    PRIMARY KEY (user_id)
)