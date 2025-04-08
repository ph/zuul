// SPDX-License-Identifier: MPL-2.0

use crate::config::Config;
use crate::fl;
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, Length, Subscription};
use cosmic::iced::widget::{column, row};
use cosmic::prelude::*;
use cosmic::widget::{self, icon, menu, nav_bar};
use cosmic::{cosmic_theme, theme};
use futures_util::SinkExt;
use std::collections::HashMap;

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    // Configuration data that persists between application runs.
    config: Config,
    passphrase: String,
    passphrase_visible: bool,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    OpenRepositoryUrl,
    SubscriptionChannel,
    UpdateConfig(Config),
    LaunchUrl(String),
    PassphraseToggleVisible,
    PassphraseChanged(String),
    Submit(String),
    Ok,
    Cancel,
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "org.heyk.Zuul";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        // Construct the app model with the runtime's core.
	let app = AppModel {
	    core,
	    passphrase: String::new(),
	    passphrase_visible: true,
	    // Optional configuration file for an application.
	    config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
		.map(|context| match Config::get_entry(&context) {
		    Ok(config) => config,
		    Err((_errors, config)) => {
			// for why in errors {
			//     tracing::error!(%why, "error loading app config");
			// }

                        config
                    }
                })
                .unwrap_or_default(),
        };

        (app, Task::none())
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// Application events will be processed through the view. Any messages emitted by
    /// events received by widgets will be passed to the update method.
    fn view(&self) -> Element<Self::Message> {
	let form = column![
	    widget::text::text(fl!("PIN")),
	    widget::secure_input("", &self.passphrase, Some(Message::PassphraseToggleVisible), false).on_input(Message::PassphraseChanged).on_submit(Message::Submit),
	    widget::text::text("DESCRIPTION"),
	    widget::icon::from_name("document-properties-symbolic"),

	    row![
		widget::button::text("Cancel").on_press(Message::Cancel),
		widget::button::text("Ok").on_press(Message::Ok),
	    ]
	];

	cosmic::widget::autosize::autosize(form, cosmic::iced::id::Id::new("autosize"))
	    .min_width(200.)
	    .min_height(150.)
	    .max_width(300.)
	    .max_height(1920.)
	    .into()
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-running async tasks running in the background which
    /// emit messages to the application through a channel. They are started at the
    /// beginning of the application, and persist through its lifetime.
    fn subscription(&self) -> Subscription<Self::Message> {
        struct MySubscription;

        Subscription::batch(vec![
            // Create a subscription which emits updates through a channel.
            Subscription::run_with_id(
                std::any::TypeId::of::<MySubscription>(),
                cosmic::iced::stream::channel(4, move |mut channel| async move {
                    _ = channel.send(Message::SubscriptionChannel).await;

                    futures_util::future::pending().await
                }),
            ),
            // Watch for application configuration changes.
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| {
                    // for why in update.errors {
                    //     tracing::error!(?why, "app config error");
                    // }

                    Message::UpdateConfig(update.config)
                }),
        ])
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::OpenRepositoryUrl => {
                _ = open::that_detached(REPOSITORY);
            }

            Message::SubscriptionChannel => {
                // For example purposes only.
            }

            Message::UpdateConfig(config) => {
                self.config = config;
            }

            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("failed to open {url:?}: {err}");
                }
            },
	    Message::PassphraseToggleVisible => self.passphrase_visible = !self.passphrase_visible,
            Message::PassphraseChanged(passphrase) => self.passphrase = passphrase,
            Message::Submit(_) => {}
            Message::Ok => {}
            Message::Cancel => {}
        }
        Task::none()
    }
}
