use clap::{Arg, App};
use preferences::{AppInfo, Preferences, PreferencesMap};
use reqwest;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

const AUTHOR: &str = "RonquilloAeon";
const API_BASE_URL: &str = "https://cryptopanic.com/api/v1";
const APP_INFO: AppInfo = AppInfo { name: "CryptoPanic Portfolio Fetcher", author: AUTHOR };
const PREFERENCES_KEY: &str = "cryptopanic-portfolio-fetcher/app";

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
    let mut file = File::create("portfolio.json").await?;
    file.write_all(data.as_bytes()).await?;

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
