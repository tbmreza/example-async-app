// #![allow(unused_imports)]
use async_std::sync::{Arc, Mutex};
use color_eyre::Result;
use myrepl::browse::*;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use thirtyfour::prelude::*;
use thirtyfour::LogType;
use thirtyfour::common::capabilities::firefox::LoggingPrefsLogLevel;
use tokio;
// use tokio::fs::File;
use tokio::fs::OpenOptions;
// use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

#[derive(Debug)]
enum Command {
    Page,
    LogTypes,
    GetLog(LogType),
    Goto,
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
                Command::Page => {
                    if let Err(e) = print_page().await {
                        eprintln!("{:?}", e);
                    }
                }
                Command::LogTypes => match driver.log_types().await {
                    Ok(log_types) => {
                        println!("{:?}", log_types)
                    }
                    Err(e) => println!("{:?}", e),
                },
                Command::GetLog(log_type) => match driver.get_log(log_type).await {
                    Ok(v) => {
                        println!("{:?}", &v)
                    }
                    Err(e) => println!("{:?}", e),
                },
                Command::Goto => {
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
                match splitted.first() {
                    Some(&"urlbar") => match splitted.get(1) {
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
                    Some(&"page") => {
                        let subcommand = splitted.get(1).unwrap_or(&"").to_string();
                        match (splitted.len(), subcommand.as_str()) {
                            (1, _) | (2, "refresh") => {
                                let tx = tx.clone();

                                tokio::spawn(async move {
                                    if subcommand == "refresh" {
                                        // TODO make this fallible: return to user prompt
                                        if let Err(_) = tx.send(Command::Goto).await {
                                            println!("receiver dropped");
                                            return;
                                        }
                                    }
                                    if let Err(_) = tx.send(Command::Page).await {
                                        println!("receiver dropped");
                                        return;
                                    }
                                });
                            }
                            _ => eprintln!("Usage: `page [refresh]`"),
                        }
                    }
                    Some(&"console-log") => {
                        if splitted.len() == 1 {
                            let tx = tx.clone();

                            tokio::spawn(async move {
                                if let Err(_) = tx.send(Command::GetLog(LogType::Browser)).await {
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
                    Some(&"log-types") => {
                        if splitted.len() == 1 {
                            let tx = tx.clone();

                            tokio::spawn(async move {
                                if let Err(_) = tx.send(Command::LogTypes).await {
                                    println!("receiver dropped");
                                    return;
                                }
                            });
                        } else {
                            println!("Wrong number of arguments: {}", line);
                        }
                    }
                    Some(&"goto") => {
                        // updates urlbar, writes to page.txt (and console.txt if any), (prints console,) then exits
                        let arg = splitted.get(1).map(|s| s.to_string());
                        match arg {
                            Some(url) if splitted.len() == 2 => {
                                let urlbar = urlbar_clone.clone();
                                *urlbar.lock().await = url;

                                let tx = tx.clone();

                                tokio::spawn(async move {
                                    if let Err(_) = tx.send(Command::Goto).await {
                                        println!("receiver dropped");
                                        return;
                                    }
                                });
                            }
                            _ => println!("Wrong number of arguments: {}", line),
                        }
                    }
                    _ => println!("Unrecognized: {}", line),
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
