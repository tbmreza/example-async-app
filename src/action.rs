/// geckodriver/marionette doesn't support get_log
// use eyre::eyre;
// use thirtyfour::error::{WebDriverError, WebDriverResult};
// use thirtyfour::prelude::*;
// use thirtyfour::LogType; // if cargo points to my fork
// use tokio::fs::OpenOptions;
// use tokio::time::{sleep, Duration};
use color_eyre::Result;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio;
use tokio::fs::File;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

pub type Urlbar = Arc<Mutex<String>>;

// problem: prompt sering ga muncul. executes after CTRL-C:
// let mut stdout = io::stdout();
// if let Ok(_) = stdout.write_all(b"dari dlm manager").await {}

/// Prints dumped `page_source` to stdout.
///
/// # Examples
///
/// ```rust
/// # use color_eyre::Result;
/// use myrepl::action::print_page;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let page_source = std::path::Path::new("tests/page.txt");
///     print_page(page_source).await?;
///     Ok(())
/// }
/// ```
pub async fn print_page(page_source: &Path) -> Result<()> {
    let buffer = {
        let mut f = File::open(page_source).await?;
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
