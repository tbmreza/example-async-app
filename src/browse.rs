/// geckodriver/marionette doesn't support get_log
use color_eyre::Result;
use eyre::eyre;
use std::sync::{Arc, Mutex};
// use thirtyfour::error::{WebDriverError, WebDriverResult};
use thirtyfour::prelude::*;
// use thirtyfour::LogType; // if cargo points to my fork
use tokio;
use tokio::fs::File;
use tokio::fs::OpenOptions;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
// use tokio::time::{sleep, Duration};

pub type Urlbar = Arc<Mutex<String>>;

pub async fn run() -> Result<()> {
    // NOTE could not set the provided `Theme` via `color_spantrace::set_theme` globally as another was already set: InstallThemeError
    // color_eyre::install()?;

    let mut caps = DesiredCapabilities::firefox();
    caps.add_firefox_arg("--headless")?;
    // let mut caps = DesiredCapabilities::chrome();
    // caps.add_chrome_arg("--headless")?;

    // NOTE timeout building driver (if driver in docker container is not safely closed?)
    let driver = WebDriver::new("http://localhost:4444", &caps).await?;

    driver.get("https://wikipedia.org").await?;

    let elem_form = driver.find_element(By::Id("search-form")).await?;
    let elem_text = elem_form.find_element(By::Id("searchInput")).await?;

    elem_text.send_keys("selenium").await?;

    let elem_button = elem_form
        .find_element(By::Css("button[type='submit']"))
        .await?;
    elem_button.click().await?;

    // Look for header to implicitly wait for the page to load.
    driver.find_element(By::ClassName("firstHeading")).await?;
    assert_eq!(driver.title().await?, "Selenium - Wikipedia");
    // assert_eq!(driver.title().await?, "jSelenium - Wikipedia");

    driver.quit().await?;
    Ok(())
}

pub async fn localhost() -> Result<()> {
    let mut caps = DesiredCapabilities::firefox();
    caps.add_firefox_arg("--headless")?;

    let driver = WebDriver::new("http://localhost:4444", &caps).await?;

    driver
        .get("http://127.0.0.1:3030/print/simplest.html")
        .await?;

    driver.find_element(By::Tag("h1")).await?;
    // assert_eq!(driver.title().await?, "Simplest HTML itw");

    let res_title = driver.title().await?;

    // if res_title == "Simplest HTML itw" {
    if res_title == "Simplest HTML itwj" {
        driver.quit().await?;
        Ok(())
    } else {
        driver.quit().await?;
        Err(eyre!("wrong title"))
    }
}

/// Driver to be shared among methods.
pub async fn new_driver(port: u16) -> Result<()> {
    let url = format!("http://localhost:{}", port);
    let mut caps = DesiredCapabilities::chrome();
    caps.add_chrome_arg("--headless")?;

    let driver = WebDriver::new(&url, &caps).await?;

    driver.quit().await?;
    Ok(())
}

#[test]
// #[ignore]
fn test_new_driver() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        if let Err(e) = new_driver(4444).await {
            println!("{}", e);
        }
    });
}

#[test]
fn test_blocking_log_types() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        if let Err(e) = log_types().await {
            println!("{}", e);
        }
    });
}

#[test]
// #[ignore]
fn test_blocking_goto() {
    let urlbar = Arc::new(Mutex::new(String::new()));
    let url = "https://wikipedia.org";
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        if let Err(e) = goto(urlbar.clone(), url).await {
            println!("{}", e);
        }
    });
}

/// Make driver if none is already running and query available log types.
pub async fn log_types() -> Result<()> {
    let mut caps = DesiredCapabilities::chrome();
    caps.add_chrome_arg("--headless")?;

    let driver = WebDriver::new("http://localhost:4444", &caps).await?;

    match driver.log_types().await {
        Ok(log_types) => {
            let expected = ["browser".to_string(), "driver".to_string()].to_vec();
            assert_eq!(log_types, expected);

            println!("{:?}", log_types)
        }
        Err(e) => println!("{:?}", e),
    }

    driver.quit().await?;
    Ok(())
}

pub async fn goto(urlbar: Urlbar, url: &str) -> Result<()> {
    let mut bar = urlbar.lock().unwrap();
    *bar = url.to_string();

    let mut caps = DesiredCapabilities::chrome();
    caps.add_chrome_arg("--headless")?;

    let driver = WebDriver::new("http://localhost:4444", &caps).await?;

    match driver.get(url).await {
        Ok(_) => {
            let source = driver.page_source().await?;

            assert_eq!("Wikipedia", driver.title().await?);

            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .truncate(true)
                .create(true)
                .open("page.txt")
                .await?;

            file.write_all(source.as_bytes()).await?;
        }
        Err(e) => println!("{:?}", e),
    }

    driver.quit().await?;
    Ok(())
}

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
