use rustyline::error::ReadlineError;
use rustyline::Editor;
use thirtyfour::prelude::*;
use tokio;

fn goto(line: &str) {
    println!("Process this: {}", line);
}

fn core_loop() {
    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let splitted: Vec<&str> = line.split(' ').filter(|x| *x != "").collect();
                if let Some(first) = splitted.first() {
                    match *first {
                        "34" => {}
                        "goto" => {
                            if splitted.len() != 2 {
                                println!("Wrong number of arguments: {}", line);
                            } else {
                                goto(&line);
                            }
                        }
                        _ => println!("Unrecognized: {}", line),
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
    rl.save_history("history.txt").unwrap();
}

fn main() {
    core_loop();
}
