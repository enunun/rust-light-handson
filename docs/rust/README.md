# Rustのノート

各Iterationで初めて使うRustの文法と概念を説明する．

| Iteration | 内容 |
| --- | --- |
| [0](iteration-00.md) | Cargo，関数，整数と`u8`，`Vec<u8>`と`&[u8]`，`String`と`format!`，外部のクレートとトレイトの`use`，テスト，モジュール，`Option`と`match`，標準入力 |
| [1](iteration-01.md) | 所有権とムーブ，借用，`Copy`と`Clone`，固定長の配列，ニュータイプパターン，`impl`とメソッド，`#[derive]`，`Display`と`FromStr`の実装，`Result`とエラーの`enum`，`chars`と`enumerate`，`const`，`into`，RustOwl |
| [2](iteration-02.md) | 属性とderiveマクロ，clap，`Path`と`PathBuf`，`std::fs`，`Write`トレイトと`impl Write`，`?`と`From`，thiserror，`pub mod`，`if let`と`matches!`，イテレーターの基本，`std::process::Command`，`tempfile`，`tests/common` |
| [3](iteration-03.md) | `Read`トレイト，`io::ErrorKind`と`match`のガード，スライスの操作，クロージャ，借用を返す関数とライフタイムの省略，`ok_or`と`map_err`，`&'static str`，`fs::read_dir`とlet chain，clapのグループ |
| [4](iteration-04.md) | 参照を持つ構造体，ライフタイム注釈と`'_`，借用する設計と所有する設計，RustOwlでライフタイムを見る，`TryFrom`と`TryInto`，スライスから配列へ，`while`，`ok_or_else` |
| [5](iteration-05.md) | `to_be_bytes`と`from_be_bytes`，`as`による整数の変換，バイト列の組み立て，読み進める型と`impl<'a>`，`BTreeMap`，`Default`と更新構文，`MetadataExt`と`PermissionsExt`，再帰と`collect`による`Result`の集約，クロージャと`filter`，`cloned` |
| [6](iteration-06.md) | `Ordering`と`sort_by`，イテレーターの`chain`と`cmp`，`then_some`，`while let`，値を受け取る関数，ジェネリクス，ゼロサイズ型，型状態パターン，`PhantomData`，`HashMap`と環境変数，`split_once`と`split_at_checked`，`SystemTime` |
| [7](iteration-07.md) | 検査済みの値だけを持つ型，`str::contains`とクロージャ，`Drop`とRAII，値を消費するメソッド，RustOwlでムーブを見る，`OpenOptions::create_new`と`fs::rename`，`OsString`，`as_deref`，`match`のフィールドのパターン |
