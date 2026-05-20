use anyhow::Result;
use tui_base_framework::App;
use tui_base_framework::examples::counter::Counter;

#[tokio::main]
async fn main() -> Result<()> {
    let counter = Counter::new();
    let mut app = App::new(counter)?;
    app.run().await?;
    Ok(())
}
