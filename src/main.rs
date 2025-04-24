use app::Args;
// SPDX-License-Identifier: {{ license }}
use tracing::info;

mod app;
mod config;
mod i18n;

fn main() -> cosmic::iced::Result {
    init_logging();

    info!("zuul ({})", <app::Zuul as cosmic::Application>::APP_ID);

    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    // Settings for configuring the application window and iced runtime.
    let settings = cosmic::app::Settings::default()
        .antialiasing(true)
        .client_decorations(true)
        .default_text_size(16.0)
        .no_main_window(true)
        .exit_on_close(true)
        .debug(true);

    // Starts the application's event loop with `()` as the application's flags.
    cosmic::app::run_single_instance::<app::Zuul>(settings, Args {})
}

fn init_logging() {
    use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    // Initialize logger
    #[cfg(feature = "console")]
    if std::env::var("TOKIO_CONSOLE").as_deref() == Ok("1") {
        std::env::set_var("RUST_LOG", "trace");
        console_subscriber::init();
    }

    let filter_layer = EnvFilter::try_from_default_env().unwrap_or(if cfg!(debug_assertions) {
        EnvFilter::new(format!("warn,{}=debug", env!("CARGO_CRATE_NAME")))
    } else {
        EnvFilter::new("warn")
    });

    let fmt_layer = fmt::layer().with_target(false);

    if let Ok(journal_layer) = tracing_journald::layer() {
        tracing_subscriber::registry()
            .with(journal_layer)
            .with(filter_layer)
            .init();
    } else {
        tracing_subscriber::registry()
            .with(fmt_layer)
            .with(filter_layer)
            .init();
    }
}
