<a href="https://discord.gg/z86rjQazq4">
    <img src="https://img.shields.io/discord/867682116586045441?color=7289DA&label=Discord&style=for-the-badge">
</a>

A discord bot that converts your <a href="https://osu.ppy.sh">osu!</a> replays into shareable video links

# Setup

In order to set this bot up, you need to have [Rust](https://www.rust-lang.org/) installed

Copy the content of `.env.example` into a new file called `.env` and supply all the variables.

Install postgresql using `sudo apt install postgresql`

Next, install `sqlx-cli` using `cargo install sqlx-cli --no-default-features --features postgres,rustls`

Migrate the database using `sqlx migrate run`. This will not work if the `DATABASE_URL` is incorrect.

Once you have done these steps, you can compile the bot using `cargo run`.
