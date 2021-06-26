use clap::{App, Arg};
use preferences::{Preferences, PreferencesMap};

mod configure;
mod consts;
mod fetch;

use consts::{APP_INFO, PREFERENCES_KEY};

fn get_preferences() -> PreferencesMap {
    let result = PreferencesMap::<String>::load(&APP_INFO, PREFERENCES_KEY);

    return match result {
        Ok(prefs) => prefs,
        Err(_) => PreferencesMap::new(),
    };
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
        configure::manage_configuration(&mut prefs, &matches)
    } else if let Some(ref matches) = matches.subcommand_matches("fetch") {
        fetch::run_fetch_portfolio(&prefs, &matches).await.unwrap()
    } else {
        println!("Please select a command to continue. Use --help to view usage.")
    }
}
