use tui_base_framework::App;
use tui_base_framework::examples::counter::Counter;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let counter = Counter::new();
    let mut app = App::new(Box::new(counter))?;
    app.run().await?;
    Ok(())
}
