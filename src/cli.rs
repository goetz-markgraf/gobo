use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
#[command(name = "gobo", about = "A shell text editor for one UTF-8 file")]
pub struct Cli {
    /// Target file to open or create on first save.
    pub path: PathBuf,
}

pub fn parse_args<I, T>(args: I) -> Result<Cli, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    Cli::try_parse_from(args)
}
