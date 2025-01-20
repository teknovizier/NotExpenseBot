use log2::*;
use std::sync::Arc;
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};
use tokio::sync::Mutex;

mod handlers;
mod utils;

use handlers::Command;
use handlers::{
    handle_amount_input, handle_category_selection, handle_subcategory_selection, help, new,
    reply_not_authorized, start, DialogueState, State,
};
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
    let state = Arc::new(Mutex::new(State::default()));

    let bot = Bot::new(&config.teloxide_token);

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(dptree::case![Command::Start].endpoint(start))
        .branch(dptree::case![Command::Help].endpoint(help))
        .branch(dptree::case![Command::New].endpoint(new));

    let handler = Update::filter_message()
        .enter_dialogue::<Message, InMemStorage<DialogueState>, DialogueState>()
        .branch(
            dptree::filter(|msg: Message, config: Config| {
                config.restrict_access && !config.allowed_users.contains(&(msg.chat.id.0 as u64))
            })
            .endpoint(reply_not_authorized),
        )
        .branch(command_handler)
        .branch(
            dptree::case![DialogueState::WaitingForCategory].endpoint(handle_category_selection),
        )
        .branch(
            dptree::case![DialogueState::WaitingForSubcategory]
                .endpoint(handle_subcategory_selection),
        )
        .branch(dptree::case![DialogueState::WaitingForAmount].endpoint(handle_amount_input));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![
            config,
            state,
            InMemStorage::<DialogueState>::new()
        ])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    info!("Stopping bot...");
}
