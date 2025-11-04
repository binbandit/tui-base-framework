use tui_base_framework::App;
use tui_base_framework::examples::progress::ProgressDemo;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let progress = ProgressDemo::new();
    let mut app = App::new(Box::new(progress))?;
    app.run().await?;
    Ok(())
}
