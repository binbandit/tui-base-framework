use tui_base_framework::App;
use tui_base_framework::examples::text_input::TextInput;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let text_input = TextInput::new();
    let mut app = App::new(Box::new(text_input))?;
    app.run().await?;
    Ok(())
}
