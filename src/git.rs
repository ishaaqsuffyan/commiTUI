use std::io::Write;
use std::process::Command;

pub fn commit_with_message(message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = tempfile::NamedTempFile::new()?;
    write!(file, "{}", message)?;
    let path = file.path();

    let status = Command::new("git")
        .arg("commit")
        .arg("-F")
        .arg(path)
        .status()?;

    if status.success() {
        println!("Commit successful!");
    } else {
        println!("Commit failed. See above for details.");
    }
    Ok(())
}