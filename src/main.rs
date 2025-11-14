mod app;
// mod auth;
mod term;
mod ui;
// mod utils;

use anyhow::Result;
// use std::sync::Arc;
// use ticks::{AccessToken, TickTick};

#[tokio::main]
async fn main() {
    // if let Some((client_id, client_secret)) = auth::get_client_id() {
    //     if let Some(access_token) = auth::get_access_token(client_id, client_secret).await {
    //         let _ = run(access_token).await;
    //     }
    // }
    let _ = run().await;
}

// async fn run(access_token: ticks::AccessToken) -> Result<()> {
//     let client = Arc::new(create_client(access_token)?);
//     let mut app = app::TickTui::new(client)?;
//     app.run().await?;
//     Ok(())
// }

async fn run() -> Result<()> {
    let mut app = app::App::new()?;
    app.run().await?;
    Ok(())
}

// fn create_client(access_token: AccessToken) -> Result<TickTick> {
//     match TickTick::new(access_token) {
//         Ok(c) => Ok(c),
//         Err(e) => {
//             auth::clear_token_cache();
//             Err(anyhow!("Failed to create TickTick client: {:?}", e))
//         }
//     }
// }
