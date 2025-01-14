use arci::Speaker;
use std::{io, process::Command};
use tracing::error;

/// A [`Speaker`] implementation using a local command.
///
/// Currently, this uses the following command:
///
/// - On macOS, use `say` command.
/// - On Windows, call [SAPI] via PowerShell.
/// - On others, use `espeak` command.
///
/// **Disclaimer**: These commands might change over time.
///
/// [SAPI]: https://en.wikipedia.org/wiki/Microsoft_Speech_API
#[derive(Debug, Default)]
#[non_exhaustive]
pub struct LocalCommand {}

impl LocalCommand {
    /// Creates a new `LocalCommand`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Similar to `Speaker::speak`, but returns an error when the command failed.
    pub fn try_speak(&self, message: &str) -> io::Result<()> {
        run_local_command(message)
    }
}

impl Speaker for LocalCommand {
    fn speak(&self, message: &str) {
        if let Err(e) = self.try_speak(message) {
            // TODO: Speaker trait seems to assume that speak method will always succeed.
            error!("{}", e);
        }
    }
}

#[cfg(not(windows))]
fn run_local_command(message: &str) -> io::Result<()> {
    #[cfg(not(target_os = "macos"))]
    const CMD_NAME: &str = "espeak";
    #[cfg(target_os = "macos")]
    const CMD_NAME: &str = "say";

    let mut cmd = Command::new(CMD_NAME);
    let status = cmd.arg(message).status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("failed to run `{}` with message {:?}", CMD_NAME, message),
        ))
    }
}

#[cfg(windows)]
fn run_local_command(message: &str) -> io::Result<()> {
    // TODO: Ideally, it would be more efficient to use SAPI directly via winapi or something.
    // https://stackoverflow.com/questions/1040655/ms-speech-from-command-line
    let cmd = format!("PowerShell -Command \"Add-Type –AssemblyName System.Speech; (New-Object System.Speech.Synthesis.SpeechSynthesizer).Speak('{}');\"", message);
    let status = Command::new("powershell").arg(cmd).status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("failed to run `powershell` with message {:?}", message),
        ))
    }
}
