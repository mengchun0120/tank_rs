use std::process;
use log::{LevelFilter, error};
use tank::mytypes::MyError;
use tank::App;
use tank::mytemplates::Settings;

const SETTING_FILE: &str = "res/settings.json";

fn main() {
    let settings = match init() {
        Ok(s) => s,
        Err(e) => {
            eprint!("Failed to initialize: {}", e);
            process::exit(1);
        }
    };

    let mut app = match App::new(settings) {
        Ok(a) => a,
        Err(e) => {
            error!("Failed to initialize app: {}", e);
            process::exit(1);
        }
    };

    app.run();
}

fn init() -> Result<Settings, MyError> {
    let settings = Settings::new(SETTING_FILE)?;
    let log_file = settings.get_str("log_file")?;

    simple_logging::log_to_file(log_file, LevelFilter::Info)?;

    Ok(settings)    
}
