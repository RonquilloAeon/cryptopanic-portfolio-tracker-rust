use clap::ArgMatches;
use preferences::PreferencesMap;

pub fn manage_configuration(
    prefs: &mut PreferencesMap,
    matches: &ArgMatches,
) -> Result<bool, std::io::Error> {
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

    return if changed { Ok(true) } else { Ok(false) };
}

pub fn list_preferences(prefs: &PreferencesMap, matches: &ArgMatches) {
    // Optionally list preferences
    if matches.is_present("list") {
        for pref in prefs.into_iter() {
            println!("{}={}", pref.0, pref.1)
        }
    }
}
