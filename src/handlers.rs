use chrono::{Datelike, Local};
use fxhash::FxHashMap;
use log2::*;
use notion_tools::structs::common::*;
use notion_tools::structs::page::*;
use notion_tools::Notion;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use teloxide::types::{KeyboardButton, KeyboardMarkup, KeyboardRemove, ParseMode};
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*, utils::command::BotCommands};
use tokio::sync::{Mutex, MutexGuard};

type MyDialogue = Dialogue<DialogueState, InMemStorage<DialogueState>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

use crate::Config;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "start the bot and show welcome message")]
    Start,
    #[command(description = "display the list of available commands")]
    Help,
    #[command(description = "add a new expense to the database")]
    New,
}

#[derive(Clone, Default)]
pub enum DialogueState {
    #[default]
    Start,
    WaitingForCategory,
    WaitingForSubcategory,
    WaitingForAmount,
}

#[derive(Clone, Default)]
pub struct State {
    selected_category: Option<String>,
    selected_subcategory: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct DatabaseInfo {
    month: u32,
    id: String,
}

// This is a temporary solution and should be replaced in future with direct fetch with Notion API
fn get_database_id() -> Result<String, Box<dyn Error>> {
    // Open the JSON file.
    let file = File::open("data.json")?;
    let reader = BufReader::new(file);

    let data: HashMap<String, Vec<DatabaseInfo>> = serde_json::from_reader(reader)?;

    // Extract the current year and month
    let now = Local::now();
    let year = now.year();
    let month = now.month();

    // Retrieve the database ID for the given year and month
    if let Some(months) = data.get(&year.to_string()) {
        if let Some(entry) = months.iter().find(|e| e.month == month) {
            Ok(entry.id.clone())
        } else {
            error!("No database found for month {} in year {}", month, year);
            Err("".into())
        }
    } else {
        error!("No databases found for year {}", year);
        Err("".into())
    }
}

fn is_empty_subcategory(subcategory: String) -> bool {
    subcategory.is_empty() || subcategory == "[EMPTY]"
}

async fn add_database_record(amount: f64, category: String, subcategory: String) -> Option<()> {
    let database_id = get_database_id().ok()?;
    let notion = Notion::new();

    let mut properties: FxHashMap<String, PageProperty> = FxHashMap::default();
    properties.insert(String::from("Amount"), PageProperty::number(amount));
    properties.insert(String::from("Category"), PageProperty::select(category));
    if !is_empty_subcategory(subcategory.clone()) {
        properties.insert(
            String::from("Subcategory"),
            PageProperty::select(subcategory),
        );
    }
    let default_comment = vec![RichText::from_str("Added by @NotExpenseBot".to_string())];
    properties.insert(
        String::from("Comment"),
        PageProperty::rich_text(default_comment),
    );
    let mut page = Page::from_properties(properties);
    page.parent.type_name = ParentType::Database;
    page.parent.database_id = Some(database_id);

    let response = notion.create_a_page(&page).await.ok()?;

    if response.status == 200 {
        info!("Item added successfully!");
        Some(())
    } else {
        error!("Received status code {}", response.status);
        None
    }
}

async fn show_categories_list(
    bot: Bot,
    chat_id: ChatId,
    categories: Vec<String>,
    title: &str,
) -> HandlerResult {
    // Create a reply keyboard with (sub)categories
    let buttons: Vec<Vec<KeyboardButton>> = categories
        .chunks(2) // Show two buttons per row
        .map(|chunk| chunk.iter().map(KeyboardButton::new).collect())
        .collect();

    let keyboard = KeyboardMarkup::new(buttons);

    // Send the message with the (sub)categories
    bot.send_message(chat_id, format!("🗂️ Choose a {}:", title))
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

async fn clear_state(mut state: MutexGuard<'_, State>) {
    state.selected_category = None;
    state.selected_subcategory = None;
}

pub async fn reply_not_authorized(bot: Bot, msg: Message) -> HandlerResult {
    warn!("Unauthorized access attempt!");
    bot.send_message(msg.chat.id, "❗ You are not authorized to use this bot.")
        .await?;
    Ok(())
}

pub async fn start(bot: Bot, msg: Message) -> HandlerResult {
    let intro_text = "<b>💰 Welcome to @NotExpenseBot!</b>\n\n\
    This bot makes it easy to track \
    and save your expenses directly to a Notion database.\n\n\
    Use /help to see available commands."
        .to_string();
    bot.send_message(msg.chat.id, intro_text)
        .parse_mode(ParseMode::Html)
        .await?;

    // Set up the bot commands menu
    bot.set_my_commands(Command::bot_commands()).await?;

    Ok(())
}

pub async fn help(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

pub async fn new(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    state: Arc<Mutex<State>>,
    config: Config,
) -> HandlerResult {
    // Clear the state
    let mut state = state.lock().await;
    state.selected_category = None;
    state.selected_subcategory = None;

    dialogue.update(DialogueState::WaitingForCategory).await?;

    bot.send_message(msg.chat.id, "➕ Let's add a new expense!")
        .await?;
    // Show the list of categories
    show_categories_list(bot, msg.chat.id, config.categories, "category").await
}

pub async fn handle_category_selection(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    state: Arc<Mutex<State>>,
    config: Config,
) -> HandlerResult {
    if let Some(category) = msg.text() {
        let categories = config.categories.clone();
        if categories.contains(&category.to_string()) {
            // Store the selected category in the state
            let mut state = state.lock().await;
            state.selected_category = Some(category.to_string());

            dialogue
                .update(DialogueState::WaitingForSubcategory)
                .await?;

            // Show the list of subcategories
            show_categories_list(bot, msg.chat.id, config.subcategories, "subcategory").await?;
        } else {
            bot.send_message(
                msg.chat.id,
                "❌ Invalid category. Please choose from the existing ones.",
            )
            .await?;
        }
    }
    Ok(())
}

pub async fn handle_subcategory_selection(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    state: Arc<Mutex<State>>,
    config: Config,
) -> HandlerResult {
    if let Some(subcategory) = msg.text() {
        let subcategories = config.subcategories.clone();
        if subcategories.contains(&subcategory.to_string()) {
            // Store the selected subcategory in the state
            let mut state = state.lock().await;
            state.selected_subcategory = Some(subcategory.to_string());

            dialogue.update(DialogueState::WaitingForAmount).await?;

            // Ask for the amount
            bot.send_message(
                msg.chat.id,
                format!(
                    "💵 Enter the expense amount in {}:",
                    config.default_currency
                ),
            )
            .reply_markup(KeyboardRemove::new()) // Remove the keyboard
            .await?;
        } else {
            bot.send_message(
                msg.chat.id,
                "❌ Invalid subcategory. Please choose from the existing ones.",
            )
            .await?;
        }
    }
    Ok(())
}

pub async fn handle_amount_input(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    state: Arc<Mutex<State>>,
    config: Config,
) -> HandlerResult {
    if let Some(amount) = msg.text() {
        let state = state.lock().await;

        let selected_category = state.selected_category.clone().unwrap_or_default();
        let selected_subcategory = state.selected_subcategory.clone().unwrap_or_default();

        // Validate the amount
        if let Ok(amount) = amount.parse::<f64>() {
            // Do not allow negative numbers
            if amount < 0.0 {
                bot.send_message(msg.chat.id, "❌ The amount cannot be negative.")
                    .await?;
                return Ok(())
            }

            let waiting_msg = bot.send_message(msg.chat.id, "⌛️").await?;

            let result = add_database_record(
                amount,
                selected_category.clone(),
                selected_subcategory.clone(),
            )
            .await;

            bot.delete_message(msg.chat.id, waiting_msg.id).await?;

            if result.is_some() {
                let message = if is_empty_subcategory(selected_subcategory.clone()) {
                    format!(
                        "✅ <b>Expense added</b>!\n\n\
                        <b>Amount</b>: {} {}\n\
                        <b>Category</b>: {}",
                        amount, config.default_currency, selected_category
                    )
                } else {
                    format!(
                        "✅ <b>Expense added</b>!\n\n\
                        <b>Amount</b>: {} {}\n\
                        <b>Category</b>: {}\n\
                        <b>Subcategory</b>: {}",
                        amount, config.default_currency, selected_category, selected_subcategory
                    )
                };
                bot.send_message(msg.chat.id, message)
                    .parse_mode(ParseMode::Html)
                    .await?;
            } else {
                bot.send_message(msg.chat.id, "❌ Error adding expense. Please try again.")
                    .await?;
            }

            // Clear the state
            clear_state(state).await;
            dialogue.exit().await?;
        } else {
            bot.send_message(msg.chat.id, "❌ Invalid amount. Please enter a number.")
                .await?;
        }
    }

    Ok(())
}
