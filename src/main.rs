#![windows_subsystem = "windows"]

use std::error::Error;

// HACK: On web target, this is a dummy entrypoint because of "[[bin]]" directive in Cargo.toml
//
// Despite of this, web entrypoint still needs to be in src/lib.rs instead. Rust throws compile
// error if this function does not exists in web targets, but still src/main.rs code will not be
// included in web targets.
fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut app = renderer_rust::App::new()?;
        app.run();
    }
    Ok(())
}
