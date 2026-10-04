use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{Args, Parser, Subcommand};

use crate::commit::{Commit, Signature, parse_offset};
use crate::error::Error;
use crate::index::Index;
use crate::object::{ObjectKind, hash_blob};
use crate::refs::RefName;
use crate::repo::Repository;
use crate::revision::resolve;
use crate::tree::parse_tree;

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
    /// Show the type, size or content of an object
    CatFile {
        #[command(flatten)]
        mode: CatFileMode,
        /// Object (HEAD, branch name or object ID)
        object: String,
    },
    /// List the entries of a tree object
    LsTree {
        /// Tree object (HEAD, branch name or object ID)
        tree: String,
    },
    /// Add file contents to the index
    Add {
        /// Files or directories to add
        #[arg(required = true)]
        paths: Vec<PathBuf>,
    },
    /// Show the files in the index
    LsFiles {
        /// Show mode and object ID of each file
        #[arg(long)]
        stage: bool,
    },
    /// Create a tree object from the index
    WriteTree,
    /// Record the index as a new commit on the current branch
    Commit {
        /// Commit message
        #[arg(short = 'm')]
        message: String,
    },
    /// Show the object ID of a revision
    RevParse {
        /// Revision (HEAD, branch name or object ID)
        rev: String,
    },
    /// List branches, or create a branch
    Branch {
        /// Name of the branch to create
        name: Option<String>,
        /// Commit the new branch points to (HEAD by default)
        start: Option<String>,
    },
    /// Create a commit object
    CommitTree {
        /// Tree object ID (4 to 40 hex digits)
        tree: String,
        /// Parent commit ID
        #[arg(short = 'p')]
        parents: Vec<String>,
        /// Commit message
        #[arg(short = 'm')]
        message: String,
    },
}

/// `cat-file`で表示するもの．どれか1つだけを指定する．
#[derive(Args)]
#[group(required = true, multiple = false)]
struct CatFileMode {
    /// Show the object type
    #[arg(short = 't')]
    kind: bool,
    /// Show the object size
    #[arg(short = 's')]
    size: bool,
    /// Show the object content
    #[arg(short = 'p')]
    pretty: bool,
}

/// コマンドラインの引数(プログラム名を除く)を解釈して実行し，結果を`out`に書く．
/// `env`は環境変数で，コミットの作者とコミッターを決めるのに使う．
pub fn run(
    args: &[&str],
    cwd: &Path,
    env: &HashMap<String, String>,
    out: &mut impl Write,
) -> Result<(), Error> {
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
        Command::CatFile { mode, object } => {
            let repo = Repository::discover(cwd)?;
            let id = resolve(&repo, &object)?;
            let (kind, content) = repo.read_object(id)?;
            if mode.kind {
                writeln!(out, "{kind}")?;
            } else if mode.size {
                writeln!(out, "{}", content.len())?;
            } else if kind == ObjectKind::Tree {
                write_tree_entries(&content, out)?;
            } else {
                out.write_all(&content)?;
            }
        }
        Command::LsTree { tree } => {
            let repo = Repository::discover(cwd)?;
            let id = resolve(&repo, &tree)?;
            let (kind, content) = repo.read_object(id)?;
            if kind != ObjectKind::Tree {
                return Err(Error::NotATree);
            }
            write_tree_entries(&content, out)?;
        }
        Command::Add { paths } => {
            Repository::discover(cwd)?.add(cwd, &paths)?;
        }
        Command::LsFiles { stage } => {
            let repo = Repository::discover(cwd)?;
            let index = Index::load(&repo.index_path())?;
            for (path, entry) in index.entries() {
                if stage {
                    writeln!(out, "{} {} 0\t{path}", entry.mode, entry.id)?;
                } else {
                    writeln!(out, "{path}")?;
                }
            }
        }
        Command::WriteTree => {
            let repo = Repository::discover(cwd)?;
            let index = Index::load(&repo.index_path())?;
            writeln!(out, "{}", repo.write_tree(&index)?)?;
        }
        Command::CommitTree {
            tree,
            parents,
            message,
        } => {
            let repo = Repository::discover(cwd)?;
            let tree = resolve(&repo, &tree)?;
            let mut builder = Commit::builder()
                .tree(tree)
                .author(signature_from_env(env, "AUTHOR")?)
                .committer(signature_from_env(env, "COMMITTER")?)
                .message(&format!("{message}\n"));
            for parent in parents {
                builder = builder.parent(resolve(&repo, &parent)?);
            }
            let commit = builder.build();
            let id = repo.write_object(ObjectKind::Commit, &commit.to_bytes())?;
            writeln!(out, "{id}")?;
        }
        Command::Commit { message } => {
            let repo = Repository::discover(cwd)?;
            let index = Index::load(&repo.index_path())?;
            let tree = repo.write_tree(&index)?;
            let head = RefName::head();
            let parent = repo.resolve_ref(&head)?;
            let mut builder = Commit::builder()
                .tree(tree)
                .author(signature_from_env(env, "AUTHOR")?)
                .committer(signature_from_env(env, "COMMITTER")?)
                .message(&format!("{message}\n"));
            if let Some(parent) = parent {
                builder = builder.parent(parent);
            }
            let id = repo.commit(builder.build())?;
            let target = repo.final_ref_name(&head)?;
            let branch = target.branch_name().unwrap_or("detached HEAD");
            let root = if parent.is_none() {
                " (root-commit)"
            } else {
                ""
            };
            let summary = message.lines().next().unwrap_or("");
            writeln!(out, "[{branch}{root} {}] {summary}", id.short())?;
        }
        Command::RevParse { rev } => {
            let repo = Repository::discover(cwd)?;
            writeln!(out, "{}", resolve(&repo, &rev)?)?;
        }
        Command::Branch { name: None, .. } => {
            let repo = Repository::discover(cwd)?;
            let current = repo.final_ref_name(&RefName::head())?;
            for branch in repo.branches()? {
                let marker = if current.branch_name() == Some(branch.as_str()) {
                    '*'
                } else {
                    ' '
                };
                writeln!(out, "{marker} {branch}")?;
            }
        }
        Command::Branch {
            name: Some(name),
            start,
        } => {
            let repo = Repository::discover(cwd)?;
            let branch = RefName::branch(&name)?;
            if repo.read_ref(&branch)?.is_some() {
                return Err(Error::BranchExists(name));
            }
            let id = resolve(&repo, start.as_deref().unwrap_or("HEAD"))?;
            repo.update_ref(&branch, id)?;
        }
    }
    Ok(())
}

/// treeの内容を解析し，エントリーを1行ずつ書く．
fn write_tree_entries(content: &[u8], out: &mut impl Write) -> Result<(), Error> {
    for entry in parse_tree(content)? {
        writeln!(out, "{entry}")?;
    }
    Ok(())
}

/// 環境変数`GIT_<role>_NAME`，`GIT_<role>_EMAIL`，`GIT_<role>_DATE`から署名を作る．
/// 時刻がなければ，今の時刻と`+0000`を使う．
fn signature_from_env(env: &HashMap<String, String>, role: &str) -> Result<Signature, Error> {
    let var = |field: &str| {
        let name = format!("GIT_{role}_{field}");
        env.get(&name).cloned().ok_or(Error::MissingVariable(name))
    };
    let name = var("NAME")?;
    let email = var("EMAIL")?;
    let (time, offset_minutes) = match var("DATE") {
        Ok(date) => parse_date(&date).ok_or(Error::InvalidDate(date))?,
        Err(_) => {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            (now.as_secs() as i64, 0)
        }
    };
    Ok(Signature {
        name,
        email,
        time,
        offset_minutes,
    })
}

/// `@<UNIX時刻> <±hhmm>`の形の時刻を読む．
fn parse_date(date: &str) -> Option<(i64, i32)> {
    let (time, offset) = date.strip_prefix('@')?.split_once(' ')?;
    Some((time.parse().ok()?, parse_offset(offset)?))
}
