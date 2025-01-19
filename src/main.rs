use log2::*;
use std::sync::Arc;
use teloxide::prelude::*;
use tokio::sync::Mutex;

mod operations;
mod utils;

use operations::Command;
use utils::Config;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file.
    // Fails if .env file not found, not readable or invalid.
    let _env = dotenvy::dotenv();

    // Read the config file
    let config = utils::load_config("config.toml");

    let _log2 = log2::open(&config.log_path)
        .module(false)
        .level("info")
        .start();

    info!("Starting bot...");

    // Initialize the state
    let state = Arc::new(Mutex::new(operations::State::default()));

    let bot = Bot::new(&config.teloxide_token);

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(dptree::case![Command::Start].endpoint(operations::start))
        .branch(dptree::case![Command::New].endpoint(operations::new));

    let handler = Update::filter_message()
        .branch(
            dptree::filter(|msg: Message, config: Config| {
                config.restrict_access && !config.allowed_users.contains(&(msg.chat.id.0 as u64))
            })
            .endpoint(operations::reply_not_authorized),
        )
        .branch(command_handler)
        .branch(
            dptree::entry()
                .filter({
                    let categories = config.categories.clone();
                    move |msg: Message| {
                        msg.text()
                            .map(|text| categories.contains(&text.to_string()))
                            .unwrap_or(false)
                    }
                })
                .endpoint(operations::handle_category_selection),
        )
        .branch(
            dptree::entry()
                .filter({
                    let subcategories = config.subcategories.clone();
                    move |msg: Message| {
                        msg.text()
                            .map(|text| subcategories.contains(&text.to_string()))
                            .unwrap_or(false)
                    }
                })
                .endpoint(operations::handle_subcategory_selection),
        )
        .branch(dptree::entry().endpoint(operations::handle_category_check_and_amount_input));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![config, state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    info!("Stopping bot...");
}
