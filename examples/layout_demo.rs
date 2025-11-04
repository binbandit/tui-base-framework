use tui_base_framework::App;
use tui_base_framework::examples::layout_demo::LayoutDemo;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let layout = LayoutDemo::new();
    let mut app = App::new(Box::new(layout))?;
    app.run().await?;
    Ok(())
}
