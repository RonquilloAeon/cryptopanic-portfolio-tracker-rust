use std::collections::HashMap;
use std::fs::create_dir;
use std::path::PathBuf;

use chrono::Utc;
use clap::ArgMatches;
use dirs::home_dir;
use preferences::PreferencesMap;
use reqwest;
use serde_json;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::consts::API_BASE_URL;
use crate::ApplicationError;

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
    data: &HashMap<String, serde_json::Value>,
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

pub fn print_portfolio_data(data: &HashMap<String, serde_json::Value>) {
    for currency in &["BTC", "USD"] {
        println!(
            "Total ({}): {}",
            currency, data["portfolio"]["totals"][currency]
        )
    }
}

pub fn get_data_dir(prefs: &PreferencesMap) -> Result<PathBuf, std::io::Error> {
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

pub async fn run_fetch_portfolio(
    prefs: &PreferencesMap,
    matches: &ArgMatches,
) -> Result<HashMap<String, serde_json::Value>, ApplicationError> {
    if !prefs.contains_key("api-token") {
        return Err(ApplicationError::NoAPIToken);
    } else {
        let data_dir = match get_data_dir(&prefs) {
            Ok(path_buf) => path_buf,
            Err(_) => return Err(ApplicationError::DirectoryError),
        };

        let results = fetch_portfolio_data(prefs.get("api-token").unwrap()).await;

        return if results.is_ok() {
            let data = process_portfolio_data(results.unwrap()).unwrap();

            if !matches.is_present("no_save") {
                save_portfolio(&data, data_dir)
                    .await
                    .map_err(|_| ApplicationError::DataSaveError)?;
            }

            Ok(data)
        } else {
            Err(ApplicationError::FetchError)
        };
    }
}
