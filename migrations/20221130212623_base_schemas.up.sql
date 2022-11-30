CREATE TABLE blacklisted_servers (
    guild_id INT8,
    reason    VARCHAR(256),
    PRIMARY KEY (guild_id)
);

CREATE TABLE blacklisted_users (
    user_id INT8,
    reason  VARCHAR(256),
    PRIMARY KEY (user_id)
);