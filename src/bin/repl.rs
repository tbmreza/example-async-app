// use async_std::task;
use color_eyre::Result;
use myrepl::browse::*;
#[allow(unused_imports)]
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::sync::{Arc, Mutex};
use thirtyfour::prelude::*;
use tokio;
use tokio::io::{self, AsyncWriteExt};
use tokio::sync::mpsc;

/// Tokio channel that starts and operates WebDriver. Accepts Method, prints response.
#[tokio::main]
async fn main() -> Result<()> {
    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    let urlbar = Arc::new(Mutex::new(String::new()));

    let driver = {
        let mut caps = DesiredCapabilities::chrome();
        caps.add_chrome_arg("--headless")?;
        WebDriver::new("http://localhost:4444", &caps).await?
    };

    let (tx, mut rx) = mpsc::channel(2);

    let manager = tokio::spawn(async move {
        while let Some(cmd) = rx.recv().await {
            match cmd {
                12 => {
                    // kalo approach ini gagal, clone contoh app client (loop) server (WebDriver)
                    println!("proses LogTypes....");
                    match driver.log_types().await {
                        Ok(log_types) => {
                            println!("{:?}", log_types)
                        }
                        Err(e) => println!("{:?}", e),
                    }
                    // problem: prompt sering ga muncul. executes after CTRL-C:
                    // let mut stdout = io::stdout();
                    // if let Ok(_) = stdout.write_all(b"dari dlm manager").await {}
                }
                22 => {
                    println!("proses goto.....");
                    // match driver.goto(url).await {
                    match driver.get("localhost:3030/print/console-log.html").await {
                        Ok(_) => {
                            // TODO actually write to page.txt
                            println!("driver get OK")
                        }
                        Err(e) => println!("{:?}", e),
                    }
                }
                cmd => println!("got: {}", cmd),
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
                            let tx = tx.clone(); // Each loop iteration moves tx.

                            tokio::spawn(async move {
                                if let Err(_) = tx.send(12).await {
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
                        match splitted.get(1) {
                            // Some(url) if splitted.len() == 2 => {
                            Some(_url) if splitted.len() == 2 => {
                                let tx = tx.clone(); // Each loop iteration moves tx.

                                tokio::spawn(async move {
                                    if let Err(_) = tx.send(22).await {
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
