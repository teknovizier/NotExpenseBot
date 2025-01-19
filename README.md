NotExpenseBot
===========================

A Telegram bot designed for tracking private expenses. Users start the bot, choose a category and subcategory, enter the amount, and the data is added to a Notion database. Currently, it's required to manually specify Notion database IDs in a JSON config file.

Currently, the bot is designed to store data in separate databases for each month.
Each database must contain the following extra fields: "Amount" (type: Number), "Category" (type: Select), "Subcategory" (type: Select) and "Comment" (type: Text).

Setting up
-------

* Create a new bot using [@Botfather](https://t.me/botfather) to get a token
* Rename `config-sample.toml` to `config.toml` and set the values for `teloxide_token`, `categories` and `subcategories`. You can also restrict access to the bot for specific Telegram users by setting `restrict_access` to `true` and specifying user Telegram IDs in `allowed_users`
* Create an integration in Notion and get [Notion API secret](https://developers.notion.com/docs/create-a-notion-integration)
* Rename `.env-sample` to `.env` and set `NOTION_API_KEY`
* Rename `data-sample.json` to `data.json` and set database IDs that can be found as written [here](https://developers.notion.com/reference/retrieve-a-database)

Usage
-------

* Run the app with `cargo run`
