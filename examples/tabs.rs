use anyhow::Result;
use tui_base_framework::App;
use tui_base_framework::examples::tabs::TabsDemo;

#[tokio::main]
async fn main() -> Result<()> {
    let tabs = TabsDemo::new();
    let mut app = App::new(tabs)?;
    app.run().await?;
    Ok(())
}
