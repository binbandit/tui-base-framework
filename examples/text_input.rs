use anyhow::Result;
use tui_base_framework::App;
use tui_base_framework::examples::text_input::TextInput;

#[tokio::main]
async fn main() -> Result<()> {
    let text_input = TextInput::new();
    let mut app = App::new(text_input)?;
    app.run().await?;
    Ok(())
}
