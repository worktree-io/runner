use anyhow::Result;

pub(super) fn prompt_editor(detected: &[(&str, &str)]) -> Result<Option<String>> {
    use std::io::{BufRead, Write};

    eprintln!("\nSelect your default editor or terminal:");
    for (i, (name, _)) in detected.iter().enumerate() {
        eprintln!("  {}. {}", i + 1, name);
    }
    let custom_idx = detected.len() + 1;
    eprintln!("  {custom_idx}. Enter a custom command");
    eprintln!("  0. Skip (no editor configured)");
    eprint!("Choice [{}]: ", if detected.is_empty() { 0 } else { 1 });
    std::io::stderr().flush().ok();

    let stdin = std::io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    let trimmed = line.trim();

    let choice: usize = if trimmed.is_empty() {
        if detected.is_empty() {
            0
        } else {
            1
        }
    } else {
        trimmed.parse().unwrap_or(usize::MAX)
    };

    if choice == 0 {
        return Ok(None);
    }
    if choice <= detected.len() {
        return Ok(Some(detected[choice - 1].1.to_string()));
    }
    if choice == custom_idx {
        eprint!("Enter editor command (e.g. \"hx .\"): ");
        std::io::stderr().flush().ok();
        let mut custom = String::new();
        stdin.lock().read_line(&mut custom)?;
        let cmd = custom.trim().to_string();
        return Ok(if cmd.is_empty() { None } else { Some(cmd) });
    }

    eprintln!("Invalid choice, skipping editor configuration.");
    Ok(None)
}
