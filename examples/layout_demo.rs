use anyhow::Result;
use tui_base_framework::App;
use tui_base_framework::examples::layout_demo::LayoutDemo;

#[tokio::main]
async fn main() -> Result<()> {
    let layout = LayoutDemo::new();
    let mut app = App::new(layout)?;
    app.run().await?;
    Ok(())
}
