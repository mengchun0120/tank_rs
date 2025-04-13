use std::process;
use tank::App;

fn main() {
    let mut app = match App::new("res/settings.json") {
        Ok(a) => a,
        Err(e) => {
            eprint!("Failed to initialize App: {}", e);
            process::exit(1);
        }
    };
    app.run();
}
