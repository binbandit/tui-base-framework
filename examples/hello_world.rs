use tui_base_framework::App;
use tui_base_framework::examples::hello_world::HelloWorld;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let hello = HelloWorld::new();
    let mut app = App::new(Box::new(hello))?;
    app.run().await?;
    Ok(())
}
