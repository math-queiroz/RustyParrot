<div align="center">
   <img src="./docs/images/logo.png" alt="Rusty-Parrot_logo.png" style="height: 120px;"/>
   <h1>RustyParrot Discord Bot</h1>
   <img alt="label-badge" src="https://img.shields.io/badge/discord-bot-green?style=flat-square"/>
   <img alt="rust-version" src="https://img.shields.io/badge/rust-1.74.0+-93450a.svg?style=flat-square"/>
</div>

## Introduction

A Rust discord bot made with [Serenity](https://github.com/serenity-rs/serenity) and [Songbird](https://github.com/serenity-rs/songbird).

## Features
+ Plays music using yt-dlp
	+ Plays from **urls**

## Installation

1. **Install Rust**: Install Rust's tools from its [official website](https://www.rust-lang.org/tools/install)

2. **Git Clone**: Clone the repository to your local machine:

   ```
   git clone https://github.com/math-queiroz/rusty-parrot.git
   ```

3. **Dependencies**: Install the required project dependencies by running:

   ```
   cargo build --release
   ```

4. **Bot Token**: If you haven't done so, head on to the [Discord Developer Portal](https://discord.com/developers/applications) to create an application and retrieve a bot token. Then add it to a `.env` file in your project's root directory. Make sure the file follow the correct naming and key pair format, e.g.:

   ```
   DISCORD_TOKEN=YOUR_BOT_TOKEN_HERE
   ```

5. **(Optional) Testing Guild**: Set a testing guild id in `.env` file for instantly testing changes in slash command interactions.

   ```
   TEST_GUILD_ID=YOUR_TESTING_GUILD_ID_HERE
   ```

6. **Bot Invite Link**: Generate an invite link for your bot by going to the OAuth2 section in the Discord Developer Portal. Make sure to select the `bot` and `applications.commands` scopes and the permissions your bot needs.

7. **Run the Bot**: Run it with the following command:

   ```
   cargo run --release
   ```

<div align="center">
   <br/>
   <small>🎉 Congrats! Your bot should now be all up and running...</small>
</div>

## Usage

To use the bot in your Discord server, use the invite link you generated in step 5. Once it's in your server, you can interact with it.

<!-- **Commands:**

+ <kbd>#TODO</kbd> - Add commands here -->

## Crates

+ dotenv v0.15.0
+ tokio v1.33.0
+ serenity v0.12
+ songbird v0.4
+ symphonia v0.5.2
+ reqwest 0.11

## Dependencies

+ [Audiopus](https://github.com/lakelezz/audiopus)
+ [Yt-dlp](https://github.com/yt-dlp/yt-dlp)

<!-- ## Contributing

Contributions are welcome! If you have improvements, bug fixes, or features to add, please create a pull request. Make sure your code follows the project's coding standards. -->

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Serenity](https://github.com/serenity-rs/serenity) - A Rust library for the Discord API.
- [Songbird](https://github.com/serenity-rs/songbird) - A library for Discord audio functionality.
