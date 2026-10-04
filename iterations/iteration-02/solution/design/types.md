# 型とモジュールの図

`rgit`のモジュール，型，公開関数を示す．
`cli`がコマンドラインの引数を解釈し，`repo`の`Repository`でオブジェクトを書き込む．

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
        }
    }
    namespace error {
        class Error {
            <<enumeration>>
            NotARepository
            Io: io::Error
            Usage: clap::Error
            +impl Display
        }
    }
    namespace object {
        class object_mod {
            <<module>>
            +blob_bytes(data: &[u8]) Vec~u8~
            +hash_blob(data: &[u8]) ObjectId
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
    cli_mod ..> Cli
    cli_mod ..> Repository
    cli_mod ..> Error
    cli_mod ..> object_mod
    Repository ..> object_mod
    Repository ..> ObjectId
    Repository ..> Error
    object_mod ..> ObjectId
    ObjectId ..> ParseObjectIdError
```

- `run`の戻り値は`Result<(), Error>`である．
- `Cli`と`Command`はclapの`Parser`と`Subcommand`を導出する．`Error`はthiserrorの`Error`を導出し，`io::Error`と`clap::Error`から`?`で変換できる．
- `Repository`は非公開のメソッド`object_path`で，IDからオブジェクトのファイルのパスを作る．
- zlibの圧縮には，外部のクレート`flate2`の`ZlibEncoder`を使う．
- `lib.rs`は`cli`を公開モジュールにし，`Error`，`hash_blob`，`ObjectId`，`ParseObjectIdError`を`pub use`で公開する．
- `main.rs`は`cli::run`を呼ぶ．`Error::Usage`ならclapのメッセージを出して終了コード2で，ほかのエラーなら`fatal: <メッセージ>`を出して終了コード128で終わる．
