#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tokio::main(flavor = "current_thread")]
async fn main() {
    rust_chip::run().await;
}
