use anyhow::Result;
use tui_base_framework::{App, examples::HelloWorld};

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = App::new(HelloWorld::new())?;
    app.run().await
}
