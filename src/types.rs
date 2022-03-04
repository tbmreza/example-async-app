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

                // TODO bedah Object di dalem message. fields (message, level, source, timestamp)
                // dan typesnya ditentuin di mana?

                write!(f, "atas. {:?}", messages)
            }
            _ => write!(f, "{}", self.0),
        }
    }
}
