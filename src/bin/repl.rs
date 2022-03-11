// TODO prepend http://
// #![allow(unused_imports)]
// use thirtyfour::common::capabilities::firefox::LoggingPrefsLogLevel;
// use thirtyfour::prelude::*;
// use tokio;
// use tokio::fs::File;
// use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
// use serde::Deserialize;
// use serde_json::{from_value, Value};
use async_std::sync::{Arc, Mutex};
use clap::StructOpt;
use color_eyre::Result;
use myrepl::action::make_driver;
use myrepl::cli::Args;
use myrepl::types::{Command, DriverMethod, LogJSON, ToCommand};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use thirtyfour::LogType;
use tokio::sync::mpsc;

/// This program consists of two big loops: a REPL and an async channel that operates WebDriver.
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    let page_txt = std::path::Path::new("page.txt");
    let log_txt = std::path::Path::new("log.txt");

    let urlbar = Arc::new(Mutex::new(String::new()));
    let urlbar_clone = urlbar.clone();

    let driver = make_driver(args.port).await?;
    let (tx, mut rx) = mpsc::channel(1);

    let _ = tokio::spawn(async move {
        while let Some(cmd) = rx.recv().await {
            use myrepl::action::*;

            match cmd {
                DriverMethod::Page => {
                    if let Err(e) = print_page(page_txt).await {
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
                        // TODO non localhost on getting log:
                        // "security - Error with Permissions-Policy header: Unrecognized feature: 'interest-cohort'."
                        use serde_json::Value;

                        let urlbar = urlbar.clone();
                        let url = urlbar.lock().await;

                        let log_json = match v {
                            Value::Array(logs) if logs.is_empty() => {
                                LogJSON(sync_from_dump(log_txt))
                            }
                            v => {
                                if sync_dump(log_txt, v.to_string()).is_err() {
                                    eprintln!("log dump failure");
                                }
                                LogJSON(v)
                            }
                        };
                        // TODO if url.is_empty, print "{log_txt} reads:"
                        println!("{} says:", &url);
                        for message in log_json.into_iter() {
                            println!("{:?}", message);
                        }
                    }
                    Err(e) => println!("{:?}", e),
                },
                DriverMethod::Goto => {
                    let urlbar = urlbar.clone();
                    let url = urlbar.lock().await;

                    match driver.get(url.clone()).await {
                        Ok(_) => match driver.page_source().await {
                            Ok(s) => {
                                if dump(s.as_bytes(), page_txt).await.is_err() {
                                    eprintln!("page_source dump failure");
                                }
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
                let splitted: Vec<&str> = line.split(' ').filter(|x| !x.is_empty()).collect();

                // Do nothing if `splitted` is empty.
                if let Some(first_word) = splitted.first() {
                    match first_word.to_command() {
                        Command::Unrecognized => {
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
                                            if tx.send(DriverMethod::Goto).await.is_err() {
                                                println!("receiver dropped");
                                                return;
                                            }
                                        }
                                        // TODO test if this runs after Goto finishes
                                        if tx.send(DriverMethod::Page).await.is_err() {
                                            println!("receiver dropped");
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
                                    if tx
                                        .send(DriverMethod::GetLog(LogType::Browser))
                                        .await
                                        .is_err()
                                    {
                                        println!("receiver dropped");
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
                                    if tx.send(DriverMethod::LogTypes).await.is_err() {
                                        println!("receiver dropped");
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
                                        if tx.send(DriverMethod::Goto).await.is_err() {
                                            println!("receiver dropped");
                                        }
                                    });
                                }
                                _ => println!("Wrong number of arguments: {}", line),
                            }
                        }
                        Command::Log => unimplemented!(),
                    }
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
