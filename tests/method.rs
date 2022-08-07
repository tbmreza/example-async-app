//! Skipping the repl interface.
// #![allow(unused_imports)]
use myrepl::action::*;
use std::env;
use thirtyfour::LogType;

/// # Invariants
///
/// - chromedriver is running on specified port
#[tokio::test]
async fn get_unavailable_log() {
    let chromedriver_port = env::var("CHROMEDRIVER_PORT")
        .unwrap_or(String::new())
        .parse::<u16>()
        .unwrap_or(4444);

    let driver = make_driver(chromedriver_port)
        .await
        .expect("chromedriver not running on this port");

    // assert 'client' not in available_logs
    let available_logs = match driver.log_types().await {
        Ok(log_types) => log_types,
        Err(e) => panic!("{:?}", e),
    };
    let client = LogType::Client;
    assert!(!available_logs.contains(&client.to_string()));

    // try anyway
    match driver.get_log(client).await {
        Ok(v) => panic!("succeed anyway: {:?}", v),
        Err(_) => println!("get_log errs, as it should"),
    };
}
#[ignore]
#[tokio::test]
async fn make_geckodriver() {
    let geckodriver_port = env::var("GECKODRIVER_PORT")
        .unwrap_or(String::new())
        .parse::<u16>()
        .unwrap_or(4445);

    match make_driver_gecko(geckodriver_port).await {
        Ok(_) => {}
        Err(e) => panic!("{:?}", e),
    }
}
// fn file_expect_statements() {
// fn receiver_dropped() {
// fn page_refresh_subcommand() {
//     assert DriverMethod::Page runs after Goto finishes

/// # Invariants
///
/// - geckodriver is running on specified port
#[ignore]
#[tokio::test]
async fn geckodriver_get_log() {
    let port = 4445_u16;
    let driver = make_driver_gecko(port)
        .await
        .expect("geckodriver not running on this port");

    // see if geckodriver/marionette supports get_log
    match driver.get_log(LogType::Browser).await {
        Ok(v) => println!("succeed despite all odds: {:?}", v),
        Err(e) => {
            driver.quit().await.unwrap();
            let tip = concat!(
                r#"get_log failed. Either wait until marionette does support it, or start looking"#,
                r#" [here](github.com/mozilla/geckodriver/issues/284#issuecomment-477677764) "#,
                r#"and code geckodriver-specific implementation."#
            );
            panic!("{}", format!("{}\n{}", e, tip));
        }
    };
    driver.quit().await.unwrap();
}
