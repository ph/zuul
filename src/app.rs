// SPDX-License-Identifier: {{ license }}

use crate::config::Config;
use crate::fl;
use cosmic::app::{context_drawer, CosmicFlags};
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::id::Id;
use cosmic::iced::{Alignment, Border, Color, Length, Shadow, Subscription};
use cosmic::iced_runtime::core::window::{Event as WindowEvent, Id as SurfaceId};
use cosmic::iced_widget::row;
use cosmic::prelude::*;
use cosmic::theme::{self, Button, Container};
use cosmic::widget::autosize;
use cosmic::widget::text::body;
use cosmic::widget::text_input::StyleSheet;
use cosmic::widget::{
    self, column, container, icon, id_container, menu, nav_bar, text_input, vertical_space, Column,
};
use cosmic::widget::{button, text};
use cosmic::{cosmic_theme, surface};
use futures_util::SinkExt;
use std::collections::HashMap;
use std::sync::LazyLock;

static AUTOSIZE_ID: LazyLock<Id> = LazyLock::new(|| Id::new("autosize"));
static MAIN_ID: LazyLock<Id> = LazyLock::new(|| Id::new("main"));

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct Zuul {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    // Configuration data that persists between application runs.
    config: Config,
    state: State,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    Ready,
    // Surface(surface::Action),

    //
    OnPassphraseChange(String),
    OnPassphraseSubmit(String),
    TogglePassphraseVisibility(bool),
}

#[derive(Debug, Clone, Default)]
pub enum State {
    #[default]
    WaitingToBeShow,
    Show,
    WaitingValidation,
}

#[derive(Debug, Clone)]
pub enum ZuulTasks {
    Open,
}

impl std::fmt::Display for ZuulTasks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZuulTasks::Open => write!(f, "open"),
        }
    }
}

pub struct Args {}

impl CosmicFlags for Args {
    type SubCommand = ZuulTasks;
    type Args = Vec<String>;
}

impl cosmic::Application for Zuul {
    type Executor = cosmic::executor::single::Executor;
    type Flags = Args;
    type Message = Message;

    const APP_ID: &'static str = "org.heyk.Zuul";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(core: cosmic::Core, flags: Self::Flags) -> (Self, cosmic::app::Task<Self::Message>) {
        let mut app = Zuul {
            state: State::WaitingToBeShow,
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
        unreachable!("No main window")
    }

    fn view_window(&self, _id: SurfaceId) -> Element<Self::Message> {
        let label_pin = text("PIN");

        let pin = text_input::secure_input("placeholder", "myvalue", None, true)
            .on_input(Message::OnPassphraseChange)
            .editing(true)
            .always_active();

        let description = text(
            r#"bonjour la famille
Toute la famille va bien
Super la vie."#,
        );

        let actions = row![button::custom(text("cancel!")), button::custom(text("OK!"))];

        let content = Column::new()
            .push(label_pin)
            .push(pin)
            .push(description.align_y(Vertical::Center))
            .push(actions);

        let window = container(id_container(content, MAIN_ID.clone()))
            .width(Length::Shrink)
            .height(Length::Shrink)
            .padding([24, 24]);

        autosize::autosize(window, AUTOSIZE_ID.clone())
            .auto_height(true)
            .min_width(300.)
            .max_width(400.)
            .max_height(1920.)
            .into()
    }

    fn update(&mut self, message: Self::Message) -> cosmic::app::Task<Self::Message> {
        match message {
            Message::Ready => Task::none(),
            Message::OnPassphraseChange(_) => Task::none(),
            Message::OnPassphraseSubmit(_) => Task::none(),
            Message::TogglePassphraseVisibility(_) => Task::none(),
        }
    }
}

impl Zuul {
    fn make_visible(&self) -> Task<Message> {
        Task::none()
    }
}
