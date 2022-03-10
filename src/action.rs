use color_eyre::Result;
use std::path::Path;
/// geckodriver/marionette doesn't support get_log
// use eyre::eyre;
// use thirtyfour::error::{WebDriverError, WebDriverResult};
// use thirtyfour::prelude::*;
// use thirtyfour::LogType; // if cargo points to my fork
// use tokio::time::{sleep, Duration};
// use std::sync::{Arc, Mutex};
//
// pub type Urlbar = Arc<Mutex<String>>;
use thirtyfour::common::capabilities::firefox::LoggingPrefsLogLevel;
use thirtyfour::prelude::*;
use thirtyfour::LogType;
use tokio;
use tokio::fs::File;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

// problem: prompt sering ga muncul. executes after CTRL-C:
// let mut stdout = io::stdout();
// if let Ok(_) = stdout.write_all(b"dari dlm manager").await {}

/// Initializes localhost driver.
pub async fn make_driver(port: u16) -> Result<WebDriver> {
    let mut caps = DesiredCapabilities::chrome();
    caps.add_chrome_arg("--headless")?;
    caps.set_logging(LogType::Browser, LoggingPrefsLogLevel::All)?;

    let server_url = format!("http://localhost:{}", port);
    Ok(WebDriver::new(&server_url, &caps).await?)
}

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

/// Dumps bytes to file.
///
/// # Examples
///
/// ```rust
/// # use color_eyre::Result;
/// use myrepl::action::dump;
/// use serde_json::{json, Value};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let j = json!(["an", "array"]);
///     let test_txt = std::path::Path::new("test.txt");
///
///     if dump(j.to_string().as_bytes(), test_txt).await.is_err() {
///         eprintln!("log dump failure");
///     }
///
///     Ok(())
/// }
/// ```
pub async fn dump(bytes: &[u8], p: &std::path::Path) -> Result<()> {
    use tokio::fs::OpenOptions;

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .create(true)
        .open(p)
        .await
        .expect("build file handle failure");

    file.write_all(bytes).await.map_err(eyre::Report::new)
}

/// Reads file as JSON value.
pub async fn from_dump(p: &std::path::Path) -> Result<serde_json::Value> {
    use serde_json::{from_slice, Value};
    use tokio::fs::OpenOptions;

    let mut file = OpenOptions::new()
        .read(true)
        // .write(true)
        .truncate(true)
        .create(true)
        .open(p)
        .await
        .expect("build file handle failure");

    let mut buf = Vec::new();

    match file.read_to_end(&mut buf).await {
        Ok(_) => {
            let j = from_slice(&buf).expect("serde conversion failure");
            Ok(j)
        }
        Err(_) => Ok(Value::Null),
    }
}
