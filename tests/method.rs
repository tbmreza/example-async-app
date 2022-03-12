// #![allow(unused_imports)]
use myrepl::action::*;
use thirtyfour::LogType;

/// Skipping the repl interface.
#[tokio::test]
async fn try_getting_unavailable_log() {
    let port = 4444_u16;
    let driver = make_driver(port)
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
// fn geckodriver_get_log() {
//     assert geckodriver/marionette doesn't support get_log
