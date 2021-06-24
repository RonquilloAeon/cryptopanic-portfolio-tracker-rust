use std::collections::HashMap;

use chrono::Utc;
use clap::{Arg, App};
use preferences::{AppInfo, Preferences, PreferencesMap};
use reqwest;
use serde_json;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

const AUTHOR: &str = "RonquilloAeon";
const API_BASE_URL: &str = "https://cryptopanic.com/api/v1";
const APP_INFO: AppInfo = AppInfo { name: "CryptoPanicPortfolioFetcher", author: AUTHOR };
const PREFERENCES_KEY: &str = "config";

async fn fetch_portfolio(api_token: &String) -> Result<String, std::io::Error> {
    let client = reqwest::Client::new();
    let data = client
        .get(format!("{}/portfolio/", API_BASE_URL))
        .query(&[("auth_token", api_token)])
        .send()
        .await.expect("Error fetching")
        .text()
        .await.expect("Error getting response text");

    Ok(data)
}

async fn save_portfolio(data: String) -> Result<(), std::io::Error> {
    // Prepare date strings
    let now = Utc::now();
    let utc_now: String = now.to_rfc3339();
    let file_name_date_part = now.format("%Y-%m-%d-%H-%M").to_string();
    let file_name = format!("{}_data.json", file_name_date_part);

    // Deserialize portfolio data
    let mut values: HashMap<String, serde_json::Value> = serde_json::from_str(&data[..])?;
    values.insert("date".to_string(), serde_json::Value::String(utc_now));

    // Save to file
    let mut file = File::create(file_name).await?;
    file.write_all(serde_json::to_string(&values).unwrap().as_bytes()).await?;

    Ok(())
}

fn get_preferences() -> PreferencesMap {
    let result = PreferencesMap::
        <String>::load(&APP_INFO, PREFERENCES_KEY);

    return match result {
        Ok(prefs) => prefs,
        Err(_) => PreferencesMap::new(),
    }
}

#[tokio::main]
async fn main() {
    let mut prefs = get_preferences();
    let matches = App::new(APP_INFO.name)
        .author(APP_INFO.author)
        .subcommand(App::new("configure")
            .about("Configure the application")
            .arg(Arg::new("api_token")
                .short('t')
                .long("apitoken")
                .value_name("API_TOKEN")
                .multiple(true)
            )
        )
        .subcommand(App::new("fetch")
            .about("Fetch your portfolio")
        )
        .get_matches();

    // Let's go!
    if let Some(ref matches) = matches.subcommand_matches("configure") {
        if matches.is_present("api_token") {
            prefs.insert("api_token".into(), matches.value_of("api_token").unwrap().into());
        }

        let result = prefs.save(&APP_INFO, PREFERENCES_KEY);
        assert!(result.is_ok());
    } else if let Some (_) = matches.subcommand_matches("fetch") {
        if !prefs.contains_key("api_token") {
            println!("Please set your API token using 'configure' command")
        } else {
            match fetch_portfolio(prefs.get("api_token").unwrap()).await {
                Ok(data) => {
                    save_portfolio(data).await.expect("Error saving to file");
                    println!("Data saved!")
                },
                Err(e) => println!("Error fetching data: {}", e)
            }
        }
    }
}
