use std::collections::HashMap;
use std::fs::create_dir;
use std::path::PathBuf;

use chrono::Utc;
use clap::{App, Arg, ArgMatches};
use dirs::home_dir;
use preferences::{AppInfo, Preferences, PreferencesMap};
use reqwest;
use serde_json;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

const AUTHOR: &str = "RonquilloAeon";
const API_BASE_URL: &str = "https://cryptopanic.com/api/v1";
const APP_INFO: AppInfo = AppInfo {
    name: "CryptoPanicPortfolioFetcher",
    author: AUTHOR,
};
const PREFERENCES_KEY: &str = "config";

async fn fetch_portfolio_data(api_token: &String) -> Result<String, std::io::Error> {
    let client = reqwest::Client::new();
    let data = client
        .get(format!("{}/portfolio/", API_BASE_URL))
        .query(&[("auth_token", api_token)])
        .send()
        .await
        .expect("Error fetching")
        .text()
        .await
        .expect("Error getting response text");

    Ok(data)
}

fn process_portfolio_data(
    data: String,
) -> Result<HashMap<String, serde_json::Value>, std::io::Error> {
    let now = Utc::now().to_rfc3339();

    // Deserialize portfolio data and add date
    let mut values: HashMap<String, serde_json::Value> = serde_json::from_str(&data[..])?;
    values.insert("date".to_string(), serde_json::Value::String(now));

    Ok(values)
}

async fn save_portfolio(
    data: HashMap<String, serde_json::Value>,
    data_dir: PathBuf,
) -> Result<(), std::io::Error> {
    // Get file name
    let file_name_date_part = Utc::now().format("%Y-%m-%d-%H-%M").to_string();
    let file_name = format!("{}_data.json", file_name_date_part);

    // Save to file
    let path = data_dir
        .as_path()
        .join(file_name)
        .to_string_lossy()
        .to_string();
    let mut file = File::create(&path).await?;
    file.write_all(serde_json::to_string(&data).unwrap().as_bytes())
        .await?;
    println!("Data saved to {}!", path);

    Ok(())
}

fn get_preferences() -> PreferencesMap {
    let result = PreferencesMap::<String>::load(&APP_INFO, PREFERENCES_KEY);

    return match result {
        Ok(prefs) => prefs,
        Err(_) => PreferencesMap::new(),
    };
}

fn get_data_dir(prefs: &PreferencesMap) -> Result<PathBuf, std::io::Error> {
    let path = match prefs.contains_key("data_dir") {
        true => PathBuf::from(prefs.get("data-dir").unwrap()),
        false => {
            let mut p = home_dir().unwrap();
            p.push("CryptoPanicData");
            p
        }
    };

    if !path.exists() {
        create_dir(&path)?;
    }

    Ok(path)
}

fn print_portfolio_data(data: &HashMap<String, serde_json::Value>) {
    for currency in &["BTC", "USD"] {
        println!(
            "Total ({}): {}",
            currency, data["portfolio"]["totals"][currency]
        )
    }
}

fn manage_configuration(prefs: &mut PreferencesMap, matches: &ArgMatches) {
    // Handle changes
    let mut changed = false;

    if matches.is_present("api_token") {
        prefs.insert(
            "api-token".into(),
            matches.value_of("api_token").unwrap().into(),
        );
        changed = true;
    }

    if matches.is_present("data_dir") {
        let data_dir = matches.value_of("data_dir").unwrap();
        prefs.insert("data-dir".into(), data_dir.into());
        changed = true;
    }

    if changed {
        let result = prefs.save(&APP_INFO, PREFERENCES_KEY);
        assert!(result.is_ok());
    }

    // Optionally list preferences
    if matches.is_present("list") {
        for pref in prefs.into_iter() {
            println!("{}={}", pref.0, pref.1)
        }
    }
}

async fn run_fetch_portfolio(
    prefs: &PreferencesMap,
    matches: &ArgMatches,
) -> Result<(), std::io::Error> {
    if !prefs.contains_key("api-token") {
        println!("Please set your API token using 'configure' command")
    } else {
        let data_dir = match get_data_dir(&prefs) {
            Ok(path_buf) => path_buf,
            Err(e) => panic!("Error selecting data dir: {}", e),
        };

        let results = fetch_portfolio_data(prefs.get("api-token").unwrap()).await;

        if results.is_ok() {
            let data = process_portfolio_data(results.unwrap()).unwrap();

            print_portfolio_data(&data);

            if !matches.is_present("no_save") {
                save_portfolio(data, data_dir)
                    .await
                    .expect("Error saving portfolio data to file");
            }
        } else {
            println!("Error fetching data: {}", results.err().unwrap())
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let mut prefs = get_preferences();
    let matches = App::new(APP_INFO.name)
        .author(APP_INFO.author)
        .subcommand(
            App::new("configure")
                .about("Configure the application")
                .arg(
                    Arg::new("data_dir")
                        .short('d')
                        .long("data-dir")
                        .value_name("DATA_DIR")
                        .takes_value(true),
                )
                .arg(
                    Arg::new("list")
                        .short('l')
                        .long("list")
                        .value_name("LIST")
                        .takes_value(false),
                )
                .arg(
                    Arg::new("api_token")
                        .short('t')
                        .long("api-token")
                        .value_name("API_TOKEN")
                        .multiple(true),
                ),
        )
        .subcommand(
            App::new("fetch").about("Fetch your portfolio").arg(
                Arg::new("no_save")
                    .short('n')
                    .long("no-save")
                    .value_name("NO_SAVE")
                    .takes_value(false),
            ),
        )
        .get_matches();

    // Let's go!
    if let Some(ref matches) = matches.subcommand_matches("configure") {
        manage_configuration(&mut prefs, &matches)
    } else if let Some(ref matches) = matches.subcommand_matches("fetch") {
        run_fetch_portfolio(&prefs, &matches).await.unwrap()
    }
}
