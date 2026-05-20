use anyhow::Result;
use tui_base_framework::App;
use tui_base_framework::examples::progress::ProgressDemo;

#[tokio::main]
async fn main() -> Result<()> {
    let progress = ProgressDemo::new();
    let mut app = App::new(progress)?;
    app.run().await?;
    Ok(())
}
