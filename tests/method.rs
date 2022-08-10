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
// fn file_expect_statements() {
// fn receiver_dropped() {
// fn page_refresh_subcommand() {
//     assert DriverMethod::Page runs after Goto finishes
