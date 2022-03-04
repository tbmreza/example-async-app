/// geckodriver/marionette doesn't support get_log
// use eyre::eyre;
// use thirtyfour::error::{WebDriverError, WebDriverResult};
// use thirtyfour::prelude::*;
// use thirtyfour::LogType; // if cargo points to my fork
// use tokio::fs::OpenOptions;
// use tokio::time::{sleep, Duration};
use color_eyre::Result;
use std::sync::{Arc, Mutex};
use tokio;
use tokio::fs::File;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

pub type Urlbar = Arc<Mutex<String>>;

// problem: prompt sering ga muncul. executes after CTRL-C:
// let mut stdout = io::stdout();
// if let Ok(_) = stdout.write_all(b"dari dlm manager").await {}
pub async fn print_page() -> Result<()> {
    let buffer = {
        let mut f = File::open("page.txt").await?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).await?;
        buffer
    };

    let mut stdout = io::stdout();
    if let Err(e) = stdout.write_all(&buffer).await {
        eprintln!("{:?}", e);
    }

    Ok(())
}
