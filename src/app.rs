// SPDX-License-Identifier: {{ license }}

use crate::config::Config;
use crate::error::ZuulErr;
use crate::fl;
use crate::subscription::read_external_commands_input;
use assuan::{Command, Response};
use cosmic::app::{context_drawer, CosmicFlags};
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::id::Id;
use cosmic::iced::platform_specific::shell::commands::{
    self,
    activation::request_token,
    layer_surface::{
        destroy_layer_surface, get_layer_surface, Anchor, KeyboardInteractivity, Layer,
    },
};
use cosmic::iced::{stream, window, Alignment, Border, Color, Length, Shadow, Subscription};
use cosmic::iced_runtime::core::layout::Limits;
use cosmic::iced_runtime::core::window::{Event as WindowEvent, Id as SurfaceId};
use cosmic::iced_runtime::platform_specific::wayland::layer_surface::SctkLayerSurfaceSettings;
use cosmic::iced_widget::row;
use cosmic::iced_winit::commands::overlap_notify::overlap_notify;
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
use futures_util::io::BufWriter;
use futures_util::{SinkExt, Stream};
use std::collections::HashMap;
use std::sync::LazyLock;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tracing::{error, info};

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
    window_id: window::Id,
    passphrase: String,
    received_commands: Vec<Command>,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    Ready,
    Surface(surface::Action),
    PinentryCommand(Command),
    Fatal,

    //
    OnPassphraseChange(String),
    OnPassphraseSubmit(String),
    TogglePassphraseVisibility(bool),
}

#[derive(Debug, Clone, Default)]
pub enum State {
    #[default]
    Waiting,
    Show,
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

    fn init(core: cosmic::Core, _flags: Self::Flags) -> (Self, cosmic::app::Task<Self::Message>) {
        let mut app = Zuul {
            state: State::Waiting,
            window_id: SurfaceId::unique(),
            passphrase: String::new(),
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
        match self.state {
            State::Waiting => row![].into(),
            State::Show => self.view_form(),
        }
    }

    fn update(&mut self, message: Self::Message) -> cosmic::app::Task<Self::Message> {
        use Message::*;

        info!(state=?self.state, message = ?message, "self.update");
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![subscribe_to_commands()])
    }
}

pub fn subscribe_to_commands() -> Subscription<Message> {
    Subscription::run_with_id(
        std::any::TypeId::of::<Command>(),
        read_external_commands_input(),
    )
    .map(|e| match e {
        Ok(c) => Message::PinentryCommand(c),
        Err(_) => Message::Fatal,
    })
}

impl Zuul {
    fn make_visible(&self) -> cosmic::app::Task<Message> {
        Task::batch(vec![
            get_layer_surface(SctkLayerSurfaceSettings {
                id: self.window_id,
                keyboard_interactivity: KeyboardInteractivity::OnDemand,
                layer: Layer::Top,
                namespace: "zuul".into(),
                size: None,
                size_limits: Limits::NONE.min_width(1.0).min_height(1.0).max_width(600.0),
                exclusive_zone: -1,
                ..Default::default()
            }),
            overlap_notify(self.window_id, true),
        ])
    }

    fn view_form(&self) -> Element<Message> {
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
            .padding([24, 24]);

        autosize::autosize(window, AUTOSIZE_ID.clone())
            .auto_height(true)
            .min_width(300.)
            .max_width(400.)
            .max_height(1920.)
            .into()
    }
}

// match message {
//     Ready => {}
//     Surface(s) => {
//         return cosmic::task::message(cosmic::Action::Cosmic(
//             cosmic::app::Action::Surface(s),
//         ));
//     }
//     _ => {}
// }
// Task::none()
