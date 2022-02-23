// use async_std::task;
use color_eyre::Result;
use myrepl::browse::*;
#[allow(unused_imports)]
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::sync::{Arc, Mutex};
// use thirtyfour::prelude::*;
use tokio;

/// Tokio channel that starts and operates WebDriver. Accepts Method, prints response.
// let driver = {
//     let mut caps = DesiredCapabilities::chrome();
//     caps.add_chrome_arg("--headless")?;
//     let d = WebDriver::new("http://localhost:4444", &caps).await?;
//     Arc::new(Mutex::new(d))
// };

// async fn core_loop() -> Result<()> {
fn core_loop() -> Result<()> {
    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    let urlbar = Arc::new(Mutex::new(String::new()));

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let splitted: Vec<&str> = line.split(' ').filter(|x| *x != "").collect();
                match splitted.first() {
                    Some(&"urlbar") => {
                        match splitted.len() {
                            1 => {
                                let url = urlbar.lock().unwrap();
                                println!("The urlbar reads: {:?}", &url);
                            }
                            _ => {
                                // sets url to shared state
                                if splitted.len() > 2 {
                                    println!("Usage: `urlbar [URL]`");
                                } else {
                                    if let Some(url) = splitted.get(1) {
                                        let mut bar = urlbar.lock().unwrap();
                                        *bar = url.to_string();
                                        // println!("set the urlbar");
                                    }
                                }
                            }
                        }
                    }
                    Some(&"page") => {
                        match splitted.len() {
                            1 => {
                                // read page.txt
                                let rt = tokio::runtime::Builder::new_current_thread()
                                    .enable_all()
                                    .build()?;
                                rt.block_on(print_page());
                            }
                            2 => {
                                // reload and then read page.txt
                                if let Some(&"refresh") = splitted.get(1) {
                                    println!("goto set url, print content of page.txt");
                                } else {
                                    println!("subcommand not recognized");
                                }
                            }
                            _ => println!("Did you mean: `page refresh`?"),
                        }
                    }
                    Some(&"console") => {
                        if splitted.len() == 1 {
                            // read console.txt
                            println!("reading...");
                        } else {
                            println!(
                                "`console` prints browser's console and doesn't take any arguments"
                            );
                        }
                    }
                    Some(&"get_log") => {
                        if let Some(url) = splitted.get(1) {
                            let rt = tokio::runtime::Builder::new_current_thread()
                                .enable_all()
                                .build()?;
                            rt.block_on(get_log(url));
                        } else {
                            println!("Wrong number of arguments: {}", line);
                        }
                    }
                    Some(&"log_types") => {
                        if splitted.len() == 1 {
                            let rt = tokio::runtime::Builder::new_current_thread()
                                .enable_all()
                                .build()?;
                            rt.block_on(async {
                                if let Err(e) = log_types().await {
                                    println!("{}", e);
                                }
                            });
                        } else {
                            println!("Wrong number of arguments: {}", line);
                        }
                    }
                    Some(&"goto") => {
                        // updates urlbar, writes to page.txt (and console.txt if any), (prints console,) then exits
                        if let Some(url) = splitted.get(1) {
                            let rt = tokio::runtime::Builder::new_current_thread()
                                .enable_all()
                                .build()?;
                            rt.block_on(goto(urlbar.clone(), url));
                        } else {
                            println!("Wrong number of arguments: {}", line);
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
    rl.save_history("history.txt").unwrap();
    Ok(())
}

fn main() -> Result<()> {
    // spawn driver in other thread here?
    // another thread to tell it to goto where

    // task::block_on(core_loop())
    core_loop()
}
