#![windows_subsystem = "windows"]

use std::error::Error;

use renderer_rust::App;

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::new()?;

    app.run();
    Ok(())
}
