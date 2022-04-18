// #![allow(unused_imports)]
use async_std::sync::{Arc, Mutex};
use clap::StructOpt;
use color_eyre::Result;
use flexi_logger::{FileSpec, Logger, WriteMode};
use if_chain::if_chain;
use lazy_static::lazy_static;
use log::*;
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
    let _logger = Logger::try_with_str("info, my::critical::module=trace")?
        .log_to_file(FileSpec::default())
        .write_mode(WriteMode::BufferAndFlush)
        .start()?;

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
                        warn!("{:?}", e);
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
                                    warn!("log dump failure");
                                }
                                LogJSON(v)
                            }
                        };

                        let heading = if url.is_empty() {
                            format!("{:?} reads:", log_txt)
                        } else {
                            format!("{} says:", &url)
                        };
                        println!("{}", &heading);
                        for message in log_json.into_iter() {
                            println!("{:?}", message);
                        }
                    }
                    Err(e) => println!("{:?}", e),
                },
                DriverMethod::Goto => {
                    let urlbar = urlbar.clone();
                    let url = urlbar.lock().await;

                    let checked_url = {
                        use regex::Regex;

                        lazy_static! {
                            static ref RE: Regex =
                                Regex::new(r"^https?:///i").expect("invalid expression");
                        }

                        let mut checked = url.clone();
                        if !RE.is_match(&checked) {
                            checked = format!("http://{}", &checked);
                        }
                        checked
                    };

                    // is better than nested match?
                    if_chain! {
                        if let Ok(_) = driver.get(checked_url).await;
                        if let Ok(source) = driver.page_source().await;
                        then {
                            if let Err(e) = dump(source.as_bytes(), page_txt).await {
                                warn!("{:?}", e);
                            }
                        }
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
                            _ => println!("Usage: `urlbar [URL]`"),
                        },
                        Command::Page => {
                            let subcommand = splitted.get(1).unwrap_or(&"").to_string();
                            match (splitted.len(), subcommand.as_str()) {
                                (1, _) | (2, "refresh") => {
                                    let tx = tx.clone();

                                    tokio::spawn(async move {
                                        if subcommand == "refresh"
                                            && tx.send(DriverMethod::Goto).await.is_err()
                                        {
                                            println!("receiver dropped");
                                            return;
                                        }

                                        if tx.send(DriverMethod::Page).await.is_err() {
                                            println!("receiver dropped");
                                        }
                                    });
                                }
                                _ => println!("Usage: `page [refresh]`"),
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
    // manager.await.unwrap();  // why is this not necessary?

    if let Err(e) = rl.save_history("history.txt") {
        warn!("{:?}", e);
    };

    Ok(())
}
