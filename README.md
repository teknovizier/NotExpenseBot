NotExpenseBot
===========================

A Telegram bot designed for tracking private expenses. Users start the bot, choose a category and subcategory, enter the amount, and the data is added to a Notion database. Currently, it's required to manually specify Notion database IDs in a JSON config file.

Currently, the bot is designed to store data in separate databases for each month.
Each database must contain the following extra fields: `Amount` (`Number`), `Category` (`Select`), `Subcategory` (`Select`) and `Comment` (`Text`).

Setting up
-------

* Create a new bot using [@Botfather](https://t.me/botfather) to get a token
* Rename `config.sample.toml` to `config.toml` and adjust the values:

| **Property**       | **Default**                                                | **Description**                                                                                                       |
|---------------------|------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------|
| `teloxide_token`   | `-`                                                        | Your Telegram bot token received from @BotFather                                                                     |
| `log_path`         | `log.txt`                                                  | The path to the bot log file                                                                                         |
| `restrict_access`  | `true`                                                     | Whether to restrict bot access to specific users only                                                                |
| `allowed_users`    | `[]`                                                       | List of Telegram user IDs allowed to access the bot. As of now, the bot is designed solely for private use, so it should include the owner's ID  |
| `categories`       | `["Category 1", "Category 2", "Category 3", "Category 4"]` | List of predefined expense categories                                                                                 |
| `subcategories`    | `["Subcategory 1", "Subcategory 2"]`                       | List of predefined expense subcategories                                                                              |
| `default_currency` | `USD`                                                      | The default currency for tracking expenses                                                                           |                                                                             |

* Create an integration in Notion and get [Notion API secret](https://developers.notion.com/docs/create-a-notion-integration)
* Rename `.env.sample` to `.env` and set `NOTION_API_KEY`
* Rename `data.sample.json` to `data.json` and set database IDs that can be found as written [here](https://developers.notion.com/reference/retrieve-a-database)

Usage
-------

* Run the app with `cargo run`
