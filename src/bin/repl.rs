#![allow(unused_imports)]
use color_eyre::Result;
use myrepl::browse::*;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::sync::{Arc, Mutex};
use thirtyfour::prelude::*;
use tokio;
use tokio::fs::File;
use tokio::fs::OpenOptions;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;

#[derive(Debug)]
enum Command {
    Page,
    LogTypes,
    Goto(String),
}

/// Tokio channel that starts and operates WebDriver. Accepts Method, prints response.
#[tokio::main]
async fn main() -> Result<()> {
    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    // let urlbar = Arc::new(Mutex::new(String::new()));

    let driver = {
        let mut caps = DesiredCapabilities::chrome();
        caps.add_chrome_arg("--headless")?;
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
                Command::Goto(url) => match driver.get(url).await {
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
                },
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
                    // Some(&"urlbar") => {
                    //     match splitted.len() {
                    //         1 => {
                    //             let url = urlbar.lock().unwrap();
                    //             println!("The urlbar reads: {:?}", &url);
                    //         }
                    //         _ => {
                    //             // sets url to shared state
                    //             if splitted.len() > 2 {
                    //                 println!("Usage: `urlbar [URL]`");
                    //             } else {
                    //                 if let Some(url) = splitted.get(1) {
                    //                     let mut bar = urlbar.lock().unwrap();
                    //                     *bar = url.to_string();
                    //                     // println!("set the urlbar");
                    //                 }
                    //             }
                    //         }
                    //     }
                    // }
                    Some(&"page") => {
                        match splitted.len() {
                            1 => {
                                let tx = tx.clone(); // Each loop iteration moves tx.

                                tokio::spawn(async move {
                                    if let Err(_) = tx.send(Command::Page).await {
                                        println!("receiver dropped");
                                        return;
                                    }
                                });
                            }
                            2 => {
                                // TODO Goto provides option to immediately print page, rather than
                                // user inputting Goto and then Page.
                                //
                                // usage: page refresh
                                // reads: Goto(shared_state_url, immediately_print = true)
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
                        if let Some(_url) = splitted.get(1) {
                            // get_log(url)
                        } else {
                            println!("Wrong number of arguments: {}", line);
                        }
                    }
                    Some(&"log_types") => {
                        if splitted.len() == 1 {
                            let tx = tx.clone(); // Each loop iteration moves tx.

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
                                let tx = tx.clone(); // Each loop iteration moves tx.

                                tokio::spawn(async move {
                                    if let Err(_) = tx.send(Command::Goto(url)).await {
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
