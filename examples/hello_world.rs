use anyhow::Result;
use tui_base_framework::App;
use tui_base_framework::examples::hello_world::HelloWorld;

#[tokio::main]
async fn main() -> Result<()> {
    let hello = HelloWorld::new();
    let mut app = App::new(hello)?;
    app.run().await?;
    Ok(())
}
