pub mod app;
pub mod config;
pub mod constants;
pub mod log;

enum ExitErrorCode {
    Config = 1,
    App = 2,
}

impl ExitErrorCode {
    fn exit(self) {
        std::process::exit(self as i32);
    }
}

#[tokio::main]
async fn main() {
    if let Err(e) = config::load() {
        errorln!("failed to initialize the config, error: {}", e.to_string());
        ExitErrorCode::Config.exit();
    }

    log::initialize();

    outputln!("starting");

    if let Err(e) = app::server().await {
        errorln!("runtime error on the app, error: {}", e.to_string());
        ExitErrorCode::App.exit();
    }
}
