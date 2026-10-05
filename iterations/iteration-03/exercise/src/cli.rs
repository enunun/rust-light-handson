use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

use crate::error::Error;
use crate::object::hash_blob;
use crate::repo::Repository;

#[derive(Parser)]
#[command(name = "rgit", about = "A Git-compatible version control tool")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create an empty repository
    Init {
        /// Directory to create the repository in
        directory: Option<PathBuf>,
    },
    /// Compute the object ID of a file
    HashObject {
        /// Write the object into the object database
        #[arg(short = 'w')]
        write: bool,
        /// File to hash
        file: PathBuf,
    },
}

/// コマンドラインの引数(プログラム名を除く)を解釈して実行し，結果を`out`に書く．
pub fn run(args: &[&str], cwd: &Path, out: &mut impl Write) -> Result<(), Error> {
    let mut argv = vec!["rgit"];
    argv.extend_from_slice(args);
    let cli = Cli::try_parse_from(argv)?;
    match cli.command {
        Command::Init { directory } => {
            let dir = match directory {
                Some(directory) => cwd.join(directory),
                None => cwd.to_path_buf(),
            };
            let repo = Repository::init(&dir)?;
            let git_dir = fs::canonicalize(repo.git_dir())?;
            writeln!(
                out,
                "Initialized empty Git repository in {}/",
                git_dir.display()
            )?;
        }
        Command::HashObject { write, file } => {
            let data = fs::read(cwd.join(file))?;
            let id = if write {
                Repository::discover(cwd)?.write_blob(&data)?
            } else {
                hash_blob(&data)
            };
            writeln!(out, "{id}")?;
        }
    }
    Ok(())
}
