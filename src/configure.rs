use clap::ArgMatches;
use preferences::{Preferences, PreferencesMap};

use crate::consts::{APP_INFO, PREFERENCES_KEY};

pub fn manage_configuration(prefs: &mut PreferencesMap, matches: &ArgMatches) {
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
