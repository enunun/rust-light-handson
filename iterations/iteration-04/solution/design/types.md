# 型とモジュールの図

`rgit`のモジュール，型，公開関数を示す．
`cli`がコマンドラインの引数を解釈し，`repo`の`Repository`でオブジェクトを読み書きする．`tree`はtreeオブジェクトの内容を解析する．

```mermaid
classDiagram
    namespace cli {
        class cli_mod {
            <<module>>
            +run(args: &[&str], cwd: &Path, out: &mut impl Write) Result
        }
        class Cli {
            <<struct>>
            -command: Command
        }
        class Command {
            <<enumeration>>
            Init: directory Option~PathBuf~
            HashObject: write bool, file PathBuf
            CatFile: mode CatFileMode, object String
            LsTree: tree String
        }
        class CatFileMode {
            <<struct>>
            -kind: bool
            -size: bool
            -pretty: bool
        }
    }
    namespace repo {
        class Repository {
            <<struct>>
            -git_dir: PathBuf
            +init(dir: &Path) Result~Repository, Error~
            +discover(start: &Path) Result~Repository, Error~
            +git_dir() &Path
            +write_blob(data: &[u8]) Result~ObjectId, Error~
            +read_object(id: ObjectId) Result
            +resolve_prefix(prefix: &str) Result~ObjectId, Error~
        }
    }
    namespace error {
        class Error {
            <<enumeration>>
            NotARepository
            ObjectNotFound: String
            AmbiguousObject: String
            NotATree
            CorruptObject: &'static str
            Io: io::Error
            Usage: clap::Error
            +impl Display
        }
    }
    namespace object {
        class object_mod {
            <<module>>
            +encode(kind: ObjectKind, data: &[u8]) Vec~u8~
            +parse_header(data: &[u8]) Result
            +hash_blob(data: &[u8]) ObjectId
        }
        class ObjectKind {
            <<enumeration>>
            Blob
            Tree
            Commit
            +impl Display
            +impl FromStr
        }
    }
    namespace tree {
        class tree_mod {
            <<module>>
            +parse_tree(data: &[u8]) Result
        }
        class Mode {
            <<enumeration>>
            File
            Executable
            Symlink
            Directory
            Submodule
            +kind() ObjectKind
            +impl Display
            +impl TryFrom~&[u8]~
        }
        class TreeEntry~'a~ {
            <<struct>>
            +mode: Mode
            +name: &'a str
            +id: ObjectId
            +impl Display
        }
    }
    namespace oid {
        class ObjectId {
            <<struct>>
            -0: [u8; 20]
            +from_bytes(bytes: [u8; 20]) ObjectId
            +short() String
            +impl Display
            +impl FromStr
        }
        class ParseObjectIdError {
            <<enumeration>>
            InvalidLength: usize
            InvalidChar: position usize, ch char
        }
    }
    Cli *-- Command
    Command *-- CatFileMode
    cli_mod ..> Cli
    cli_mod ..> Repository
    cli_mod ..> Error
    cli_mod ..> object_mod
    Repository ..> object_mod
    cli_mod ..> tree_mod
    tree_mod ..> TreeEntry
    TreeEntry *-- Mode
    TreeEntry *-- ObjectId
    Mode ..> ObjectKind
    Repository ..> ObjectId
    Repository ..> ObjectKind
    Repository ..> Error
    object_mod ..> ObjectId
    object_mod ..> ObjectKind
    object_mod ..> Error
    ObjectId ..> ParseObjectIdError
```

- `run`の戻り値は`Result<(), Error>`である．
- `read_object`の戻り値は`Result<(ObjectKind, Vec<u8>), Error>`，`parse_header`の戻り値は`Result<(ObjectKind, &[u8]), Error>`である．`parse_header`が返す内容は，引数のバイト列の一部を借りたものである．
- `Cli`と`Command`はclapの`Parser`と`Subcommand`を導出する．`CatFileMode`は`Args`を導出し，`-t`，`-s`，`-p`のどれか1つだけを受け付ける．`Error`はthiserrorの`Error`を導出し，`io::Error`と`clap::Error`から`?`で変換できる．
- `parse_tree`の戻り値は`Result<Vec<TreeEntry<'_>>, Error>`である．
- `TreeEntry`の`name`は，`parse_tree`に渡したバイト列の一部を借りている．`TreeEntry`は，そのバイト列より長く使えない．
- `cli`は非公開の関数`write_tree_entries`で，treeのエントリーを1行ずつ書く．`cat-file -p`と`ls-tree`の両方が使う．
- `Repository`は非公開のメソッド`object_path`で，IDからオブジェクトのファイルのパスを作る．
- zlibの圧縮には，外部のクレート`flate2`の`ZlibEncoder`を使う．
- `lib.rs`は`cli`を公開モジュールにし，`Error`，`hash_blob`，`ObjectId`，`ParseObjectIdError`を`pub use`で公開する．
- `main.rs`は`cli::run`を呼ぶ．`Error::Usage`ならclapのメッセージを出して終了コード2で，ほかのエラーなら`fatal: <メッセージ>`を出して終了コード128で終わる．
