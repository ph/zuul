// SPDX-License-Identifier: {{ license }}

use crate::config::Config;
use crate::fl;
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::cosmic_theme;
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::id::Id;
use cosmic::iced::{Alignment, Border, Color, Length, Shadow, Subscription};
use cosmic::prelude::*;
use cosmic::theme::{self, Button, Container};
use cosmic::widget::text_input::StyleSheet;
use cosmic::widget::{
    self, autosize, column, container, icon, id_container, menu, nav_bar, text_input,
    vertical_space, Column,
};
use futures_util::SinkExt;
use std::collections::HashMap;
use std::sync::LazyLock;

static AUTOSIZE_ID: LazyLock<Id> = LazyLock::new(|| Id::new("autosize"));
static MAIN_ID: LazyLock<Id> = LazyLock::new(|| Id::new("main"));

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    // Configuration data that persists between application runs.
    config: Config,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    Hello,
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
        let mut app = AppModel {
            core,
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

    fn view(&self) -> Element<Self::Message> {
        let pin = text_input("placeholder", "yoodi").style(cosmic::theme::TextInput::Custom {
            active: Box::new(|theme| theme.focused(&cosmic::theme::TextInput::Inline)),
            error: Box::new(|theme| theme.focused(&cosmic::theme::TextInput::Inline)),
            hovered: Box::new(|theme| theme.focused(&cosmic::theme::TextInput::Inline)),
            focused: Box::new(|theme| theme.focused(&cosmic::theme::TextInput::Inline)),
            disabled: Box::new(|theme| theme.disabled(&cosmic::theme::TextInput::Inline)),
        });

        let content = widget::column()
            .push(pin)
            .max_width(600)
            .width(Length::Shrink)
            .height(Length::Shrink)
            .spacing(16);

        let window = Column::new()
            .push(vertical_space().height(Length::Fixed(16.)))
            .push(
                container(id_container(content, MAIN_ID.clone()))
                    .width(Length::Shrink)
                    .height(Length::Shrink)
                    .class(Container::Custom(Box::new(|theme| container::Style {
                        text_color: Some(theme.cosmic().on_bg_color().into()),
                        icon_color: Some(theme.cosmic().on_bg_color().into()),
                        background: Some(Color::from(theme.cosmic().background.base).into()),
                        border: Border {
                            radius: theme.cosmic().corner_radii.radius_m.into(),
                            width: 1.0,
                            color: theme.cosmic().bg_divider().into(),
                        },
                        shadow: Shadow::default(),
                    })))
                    .padding([24, 32]),
            );

        Element::from(autosize::autosize(window, AUTOSIZE_ID.clone()))
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        Task::none()
    }
}
