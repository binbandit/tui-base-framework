use tui_base_framework::App;
use tui_base_framework::examples::list_selector::ListSelector;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let list = ListSelector::new();
    let mut app = App::new(Box::new(list))?;
    app.run().await?;
    Ok(())
}
