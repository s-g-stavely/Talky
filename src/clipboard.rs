use anyhow::{Context, Result};
use clipboard_rs::{Clipboard, ClipboardContext, ContentFormat};
use enigo::{Enigo, Key, Keyboard, Direction};
use std::thread;
use std::time::Duration;

/// Copies text to the clipboard
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    let ctx = ClipboardContext::new().unwrap();
    ctx.set_text(text.to_string()).map_err(|e| anyhow::anyhow!("Failed to set clipboard text: {}", e))
}

/// Pastes the current clipboard content using keyboard emulation
pub fn paste_clipboard() -> Result<()> {
    // todo better error handling
    let mut enigo = Enigo::new(&enigo::Settings::default())?;
    
    // Small delay to ensure applications are ready
    thread::sleep(Duration::from_millis(500));
    
    // Execute Ctrl+V (or Command+V on macOS)
    #[cfg(target_os = "macos")]
    {
        enigo.key_down(Key::Meta);
        enigo.key_click(Key::Layout('v'));
        enigo.key_up(Key::Meta);
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        enigo.key(Key::Control, Direction::Press);
        enigo.key(Key::V, Direction::Click);
        enigo.key(Key::Control, Direction::Release);
    }
    
    Ok(())
}

/// Copies text to clipboard and then pastes it
pub fn copy_and_paste(text: &str) -> Result<()> {
    println!("Copying to clipboard: {}", text);
    copy_to_clipboard(text)?;
    
    println!("Pasting clipboard contents...");
    paste_clipboard()?;
    
    Ok(())
}
