use serde::Deserialize;
use serde_json::{from_value, Value};
use strum_macros::EnumIter;
use thirtyfour::LogType;

#[derive(Debug)]
pub enum DriverMethod {
    Page,
    LogTypes,
    GetLog(LogType),
    Goto,
}

#[derive(Debug, EnumIter, PartialEq)]
pub enum Command {
    Goto,
    Urlbar,
    Page,
    LogTypes,
    Log,
    ConsoleLog,
    Unrecognized,
}

pub trait ToCommand {
    fn to_command(&self) -> Command;
}

impl ToCommand for &str {
    fn to_command(&self) -> Command {
        let input = self.to_lowercase();
        match input.trim() {
            "goto" => Command::Goto,
            "urlbar" => Command::Urlbar,
            "page" => Command::Page,
            "log-types" | "lt" => Command::LogTypes,
            "log" => Command::Log,
            "console-log" | "cl" => Command::ConsoleLog,
            _ => Command::Unrecognized,
        }
    }
}

#[derive(Deserialize, Default)]
struct ConsoleItem {
    message: String,
    #[allow(dead_code)]
    level: String,
    #[allow(dead_code)]
    source: String,
    #[allow(dead_code)]
    timestamp: u64,
}

/// Formats serialized json to vector of messages, our field of interest.
///
/// Unfortunately, message is pre-formatted to String by the protocol. There is nothing we could do
/// to retrieve the lost information. Otherwise, displaying a JavaScript object would be possible.

#[derive(Default)]
pub struct LogJSON(pub Value);

use std::iter::IntoIterator;
impl IntoIterator for LogJSON {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let selenium_value = &(self.0);

        match from_value(selenium_value.to_owned()).unwrap_or(Value::Null) {
            Value::Array(items) => {
                let console_items = items
                    .into_iter()
                    .map(|v| from_value::<ConsoleItem>(v).unwrap_or_default());

                let messages = console_items
                    .into_iter()
                    .map(|i| behead(i.message))
                    .collect::<Vec<String>>();

                messages.into_iter()
            }
            _ => Vec::new().into_iter(),
        }
    }
}

/// First word of ConsoleItem's message is the URL. Also trims double-quotes around the JavaScript value.
fn behead(message: String) -> String {
    let mut words = message
        .split(' ')
        .filter(|x| !x.is_empty())
        .skip(1)
        .map(|word| word.trim_matches('"'));

    match words.next() {
        None => String::new(),
        Some(word) => {
            let mut message = format!("[{}]", word);

            for word in words {
                message = format!("{} {}", message, word);
            }
            message
        }
    }
}

#[test]
fn test_behead() {
    println!(
        "{:?}",
        behead("http://tarrasque.dmp.loc/ 75:20 \"aha\"".to_string())
    );
}

#[test]
fn test_log_json_iter() {
    for message in LogJSON::default().into_iter() {
        println!("{:?}", message);
    }
}
