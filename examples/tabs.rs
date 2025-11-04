use tui_base_framework::App;
use tui_base_framework::examples::tabs::TabsDemo;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let tabs = TabsDemo::new();
    let mut app = App::new(Box::new(tabs))?;
    app.run().await?;
    Ok(())
}
