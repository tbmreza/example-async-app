// #![allow(unused_imports)]
use async_std::sync::{Arc, Mutex};
use color_eyre::Result;
use myrepl::browse::*;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use thirtyfour::common::capabilities::firefox::LoggingPrefsLogLevel;
use thirtyfour::prelude::*;
use thirtyfour::LogType;
use tokio;
// use tokio::fs::File;
use tokio::fs::OpenOptions;
// use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use serde::Deserialize;
use serde_json::{from_value, Value};
use strum_macros::EnumIter;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

#[derive(Debug)]
enum DriverMethod {
    Page,
    LogTypes,
    GetLog(LogType),
    Goto,
}

#[derive(Debug, EnumIter, PartialEq)]
enum Command {
    Help,
    Goto,
    Urlbar,
    Page,
    LogTypes,
    Log,
    ConsoleLog,
    Unrecognized,
}

trait ToCommand {
    fn to_command(&self) -> Command;
}

// fn pascal_to_kebab
// Command::iter().map(pascal_to_kebab)

// use self::Command::*;
// impl Command {
//     pub fn iter() -> std::slice::Iter<'static, Command> {
//         static COMMANDS: [Command; 8] = [
//             Help,
//             Goto,
//             Urlbar,
//             Page,
//             LogTypes,
//             Log,
//             ConsoleLog,
//             Unrecognized,
//         ];
//         COMMANDS.iter()
//     }
// }

impl ToCommand for &str {
    fn to_command(&self) -> Command {
        let input = self.to_lowercase();
        match input.trim() {
            "help" => Command::Help,
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

struct LogJSON(Value);

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
        let selenum_value = &self.clone().0;

        match from_value(selenum_value.to_owned()).unwrap_or(Value::Null) {
            Value::Array(items) => {
                let console_items = items
                    .into_iter()
                    .map(|v| from_value::<ConsoleItem>(v).unwrap_or(ConsoleItem::default()))
                    .collect::<Vec<ConsoleItem>>();

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

/// This program consists of two big loops: a REPL and an async channel that operates WebDriver.
#[tokio::main]
async fn main() -> Result<()> {
    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    let urlbar = Arc::new(Mutex::new(String::new()));
    let urlbar_clone = urlbar.clone();

    let driver = {
        let mut caps = DesiredCapabilities::chrome();
        caps.add_chrome_arg("--headless")?;
        caps.set_logging(LogType::Browser, LoggingPrefsLogLevel::All)?;
        WebDriver::new("http://localhost:4444", &caps).await?
    };

    let (tx, mut rx) = mpsc::channel(1);

    let _ = tokio::spawn(async move {
        while let Some(cmd) = rx.recv().await {
            match cmd {
                DriverMethod::Page => {
                    if let Err(e) = print_page().await {
                        eprintln!("{:?}", e);
                    }
                }
                DriverMethod::LogTypes => match driver.log_types().await {
                    Ok(log_types) => {
                        println!("{:?}", log_types)
                    }
                    Err(e) => println!("{:?}", e),
                },
                DriverMethod::GetLog(log_type) => match driver.get_log(log_type).await {
                    Ok(v) => {
                        println!("{:?}", &LogJSON(v).to_string())
                    }
                    Err(e) => println!("{:?}", e),
                },
                DriverMethod::Goto => {
                    let urlbar = urlbar.clone();
                    let url = urlbar.lock().await;

                    match driver.get(url.clone()).await {
                        Ok(_) => match driver.page_source().await {
                            Ok(s) => {
                                let mut file = OpenOptions::new()
                                    .read(true)
                                    .write(true)
                                    .truncate(true)
                                    .create(true)
                                    .open("page.txt")
                                    .await
                                    .expect("build file handle failure");

                                if let Err(e) = file.write_all(s.as_bytes()).await {
                                    eprintln!("{:?}", e);
                                };
                            }
                            Err(e) => eprintln!("{:?}", e),
                        },
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
            }
        }
    });

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let splitted: Vec<&str> = line.split(' ').filter(|x| *x != "").collect();
                let first_word = splitted.first().unwrap_or(&"");

                match first_word.to_command() {
                    Command::Help | Command::Unrecognized => {
                        if splitted.len() == 1 {
                            use case_style::CaseStyle;
                            use strum::IntoEnumIterator;

                            let kebabcase_commands = Command::iter()
                                .filter(|c| *c != Command::Unrecognized)
                                .map(|c| {
                                    let command = format!("{:?}", c);
                                    CaseStyle::from_pascalcase(command).to_kebabcase()
                                });

                            println!("Available commands:");
                            for command in kebabcase_commands {
                                println!("{:?}", command);
                            }
                        } else {
                            println!(
                                "`{}` prints available commands and doesn't take any arguments.",
                                format!("{:?}", Command::Help).to_lowercase()
                            );
                        }
                    }
                    Command::Urlbar => match splitted.get(1) {
                        None => {
                            let urlbar = urlbar_clone.clone();
                            let url = urlbar.lock().await;

                            println!("The urlbar reads: {:?}", &url.clone());
                        }
                        Some(arg) if splitted.len() == 2 => {
                            let urlbar = urlbar_clone.clone();
                            *urlbar.lock().await = arg.to_string();
                        }
                        _ => eprintln!("Usage: `urlbar [URL]`"),
                    },
                    Command::Page => {
                        let subcommand = splitted.get(1).unwrap_or(&"").to_string();
                        match (splitted.len(), subcommand.as_str()) {
                            (1, _) | (2, "refresh") => {
                                let tx = tx.clone();

                                tokio::spawn(async move {
                                    if subcommand == "refresh" {
                                        // TODO make this fallible: return to user prompt
                                        if let Err(_) = tx.send(DriverMethod::Goto).await {
                                            println!("receiver dropped");
                                            return;
                                        }
                                    }
                                    if let Err(_) = tx.send(DriverMethod::Page).await {
                                        println!("receiver dropped");
                                        return;
                                    }
                                });
                            }
                            _ => eprintln!("Usage: `page [refresh]`"),
                        }
                    }
                    Command::ConsoleLog => {
                        if splitted.len() == 1 {
                            let tx = tx.clone();

                            tokio::spawn(async move {
                                if let Err(_) =
                                    tx.send(DriverMethod::GetLog(LogType::Browser)).await
                                {
                                    println!("receiver dropped");
                                    return;
                                }
                            });
                        } else {
                            println!(
                                "`console` prints browser's console and doesn't take any arguments"
                            );
                        }
                    }
                    Command::LogTypes => {
                        if splitted.len() == 1 {
                            let tx = tx.clone();

                            tokio::spawn(async move {
                                if let Err(_) = tx.send(DriverMethod::LogTypes).await {
                                    println!("receiver dropped");
                                    return;
                                }
                            });
                        } else {
                            println!("Wrong number of arguments: {}", line);
                        }
                    }
                    Command::Goto => {
                        // updates urlbar, writes to page.txt (and console.txt if any), (prints console,) then exits
                        let arg = splitted.get(1).map(|s| s.to_string());
                        match arg {
                            Some(url) if splitted.len() == 2 => {
                                let urlbar = urlbar_clone.clone();
                                *urlbar.lock().await = url;

                                let tx = tx.clone();

                                tokio::spawn(async move {
                                    if let Err(_) = tx.send(DriverMethod::Goto).await {
                                        println!("receiver dropped");
                                        return;
                                    }
                                });
                            }
                            _ => println!("Wrong number of arguments: {}", line),
                        }
                    }
                    Command::Log => unimplemented!(),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    // manager.await.unwrap();  // dengan atau tanpa ini: kadang promptnya gak ada, tapi functionality ok

    rl.save_history("history.txt").unwrap();
    Ok(())
}
