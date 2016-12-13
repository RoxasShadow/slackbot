use std::collections::HashMap;

use slack::{Error, Event, EventHandler, RtmClient};

use serde_json;
use regex::Regex;

use super::CommandHandler;
use super::sender::Sender;

pub struct SlackBotEventHandler<'a> {
    bot_name: String,
    handlers: &'a mut HashMap<String, Box<CommandHandler>>
}

#[derive(Debug, PartialEq, Deserialize)]
struct SlackEvent {
    #[serde(rename = "type")]
    event_type: Option<String>,
    subtype: Option<String>,
    #[serde(rename = "user")]
    user_id: Option<String>,
    text: Option<String>,
    channel: Option<String>
}

impl<'a> SlackBotEventHandler<'a> {
    pub fn new<S: Into<String>>(name: S, handlers: &'a mut HashMap<String, Box<CommandHandler>>) -> Self {
        SlackBotEventHandler {
            bot_name: name.into(),
            handlers: handlers
        }
    }
}

impl<'a> EventHandler for SlackBotEventHandler<'a> {
    fn on_event(&mut self, cli: &mut RtmClient, _: Result<Event, Error>, json_str: &str) {
        info!("{}", json_str);

        let event: SlackEvent = serde_json::from_str(json_str).unwrap();

        if event.event_type == Some("message".to_owned()) && event.subtype == None {
            let user_id = event.user_id.unwrap();
            let user = cli.get_users().iter().find(|u| u.id == user_id).unwrap().to_owned();
            let text = event.text.unwrap();
            let channel = event.channel.unwrap();

            for (command_name, handler) in self.handlers.into_iter() {
                let regex = Regex::new(command_name).unwrap();
                if regex.is_match(&text) {
                    let mut sender = Sender::new(cli, channel.to_owned(), user.to_owned());
                    handler.handle(&mut sender, &regex.captures(&text).unwrap());
                }
            }
        }
    }

    fn on_ping(&mut self, _: &mut RtmClient) {}

    fn on_close(&mut self, _: &mut RtmClient) {}

    fn on_connect(&mut self, _: &mut RtmClient) {}
}
