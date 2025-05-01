// SPDX-FileCopyrightText: 2025 Pier-Hugues Pellerin <ph@heykimo.com>
//
// SPDX-License-Identifier: MIT

use std::borrow::Cow;

use assuan::Command;

#[derive(Default, Clone, Debug)]
pub struct Form {
    prompt: String,
    button_ok: String,
    button_cancel: String,
    description: Option<String>,
}

impl Form {
    pub fn prompt(&self) -> Cow<str> {
        Cow::Borrowed(&self.prompt)
    }

    pub fn button_ok(&self) -> Cow<str> {
        Cow::Borrowed(&self.button_ok)
    }

    pub fn button_cancel(&self) -> Cow<str> {
        Cow::Borrowed(&self.button_cancel)
    }

    pub fn description(&self) -> Option<Cow<str>> {
        self.description.as_deref().map(Cow::Borrowed)
    }
}

struct FormBuilder {
    prompt: String,
    button_ok: String,
    button_cancel: String,
    description: Option<String>,
}

impl FormBuilder {
    fn new() -> Self {
        Self {
            prompt: String::from("PIN:"),
            button_ok: String::from("OK"),
            button_cancel: String::from("cancel"),
            description: None,
        }
    }

    fn with_prompt(mut self, s: impl Into<String>) -> Self {
        self.prompt = s.into();
        self
    }

    fn with_button_ok(mut self, s: impl Into<String>) -> Self {
        self.button_ok = s.into();
        self
    }

    fn with_button_cancel(mut self, s: impl Into<String>) -> Self {
        self.button_cancel = s.into();
        self
    }

    fn with_description(mut self, s: impl Into<String>) -> Self {
        self.description = Some(s.into());
        self
    }

    fn build(self) -> Form {
        Form {
            prompt: self.prompt,
            button_ok: self.button_ok,
            button_cancel: self.button_cancel,
            description: self.description,
        }
    }
}

pub fn apply_commands(commands: &[Command]) -> Form {
    let mut b = FormBuilder::new();

    for command in commands {
        // iteratively building the form.
        b = match command {
            Command::SetPrompt(p) => b.with_prompt(p),
            Command::SetOk(t) => b.with_button_ok(t),
            Command::SetCancel(t) => b.with_button_cancel(t),
            Command::SetDesc(t) => b.with_description(t),
            _ => continue, // ignore unsupported commands for now.
        };
    }
    b.build()
}
