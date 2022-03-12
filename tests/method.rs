#![allow(unused_imports)]
use myrepl::action::*;
use myrepl::types::DriverMethod::*;

/// Skipping the repl interface.
#[tokio::test]
async fn try_getting_unavailable_log() {
    // make_driver
    let port = 4444_u16;
    let driver = make_driver(port).await.unwrap();

    // get available logs
    let available_logs = match driver.log_types().await {
        Ok(log_types) => log_types,
        Err(e) => panic!("{:?}", e),
    };

    // assert 'client' not in available_logs
    let client = String::from("client");
    assert!(!available_logs.contains(&client));

    // try anyway
    match driver.get_log(client.into()).await {
        Ok(v) => println!("succeed anyway: {:?}", v),
        Err(e) => println!("errs, as it should: {:?}", e),
    };
}
// fn file_expect_statements() {
// fn receiver_dropped() {
