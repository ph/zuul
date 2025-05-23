// SPDX-FileCopyrightText: 2025 Pier-Hugues Pellerin <ph@heykimo.com>
//
// SPDX-License-Identifier: MIT

use crate::error::ZuulErr;
use crate::form::Form;
use crate::form::apply_commands;
use assuan::{Command, Response};
use cosmic::iced::stream;
use futures_util::SinkExt;
use futures_util::Stream;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::{BufReader, BufWriter};

#[derive(Clone, Debug)]
pub enum Event {
    Bye,
    Form(Form),
}

pub fn read_external_commands_input() -> impl Stream<Item = Result<Event, ZuulErr>> {
    stream::try_channel(1, async move |mut output| {
        let mut commands = Vec::new();

        let stdin = tokio::io::stdin();
        let buf = BufReader::new(stdin);
        let mut lines = buf.lines();

        let mut stdout = tokio::io::stdout();

        let mut w = BufWriter::new(&mut stdout);

        let mut reply = async move |m: Response| {
            w.write_all(&format!("{}\n", m.to_pinentry()).into_bytes())
                .await
                .unwrap();
            w.flush().await.unwrap();
        };

        reply(Response::OkHello).await;

        // NOTE(ph): add handle bad case.
        while let Some(line) = lines.next_line().await? {
            // info!("line received: `{}`", line);
            let command = Command::try_from(line)?;
            // info!("command extracted: `{:?}`", command);

            match command {
                Command::Bye => {
                    let _ = output.send(Event::Bye).await;
                    reply(Response::Ok).await;
                    return Ok(());
                }
                Command::GetPin => {
                    let form = apply_commands(&commands);
                    let _ = output.send(Event::Form(form)).await;
                }
                _ => {
                    commands.push(command);
                    reply(Response::Ok).await;
                }
            }
        }

        Ok(())
    })
}
