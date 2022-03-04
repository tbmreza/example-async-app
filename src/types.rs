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

pub struct LogJSON(pub Value);

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
impl std::fmt::Display for LogJSON {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let selenium_value = &(self.0);

        match from_value(selenium_value.to_owned()).unwrap_or(Value::Null) {
            Value::Array(items) => {
                let console_items = items
                    .into_iter()
                    .map(|v| from_value::<ConsoleItem>(v).unwrap_or_default());

                let messages = console_items
                    .into_iter()
                    .map(|i| i.message)
                    .collect::<Vec<String>>();

                write!(f, "{:?}", messages)
            }
            _ => write!(f, "{}", self.0),
        }
    }
}
