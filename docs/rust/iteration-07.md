# Iteration 7：検査済みの型と`Drop`

Iteration 7では，参照とブランチを扱い，`rgit commit`でブランチを進める．
このノートでは，作るときに検査した値だけを持つニュータイプ，`Drop`による後片付け(RAII)，`self`を受け取って値を消費するメソッド，ファイルの排他的な作成と名前の変更を説明する．

## 検査済みの値だけを持つ型

参照名には規則がある(`refs/`で始まる，`..`や空白を含まない，など)．
参照名を`String`のまま扱うと，参照を読む関数と書く関数の両方が，受け取るたびに規則を調べる必要がある．
調べ忘れた関数があると，`../../etc/passwd`のような名前で`.git`の外のファイルを書き換えてしまう．

作るときに一度だけ検査し，検査に通った値だけを持つ型を作ると，この問題を型で防げる．

```rust
pub struct RefName(String);

impl TryFrom<&str> for RefName {
    type Error = Error;

    fn try_from(name: &str) -> Result<RefName, Error> {
        if name == "HEAD" || is_valid_ref_path(name) {
            Ok(RefName(name.to_string()))
        } else {
            Err(Error::InvalidRefName(name.to_string()))
        }
    }
}
```

フィールドは非公開なので，モジュールの外で`RefName`を作るには`try_from`を通るしかない．
`&RefName`を受け取る関数は，規則を満たした名前であることを前提にできる．
Iteration 1の`ObjectId`も，40桁の16進数を検査してから作る型だった．
「受け取ったものを検査する(validate)」のではなく「検査済みの型に変換する(parse)」設計は，Rustでよく使われる．

`Display`を実装すれば`{}`で表示でき，`as_str()`で中の`&str`を読ませられる．中身を書き換えるメソッドは用意しない．

### 文字の検査

`str::contains`には，文字列だけでなく，`char`を受け取って`bool`を返すクロージャも渡せる．

```rust
let forbidden = |ch: char| ch.is_ascii_control() || " ~^:?*[\\".contains(ch);
assert!("refs/heads/bad name".contains(forbidden));
assert!(!"refs/heads/main".contains(forbidden));
```

`split('/')`は区切りで分けたイテレーターを返す．`all(|part| …)`は，すべての要素が条件を満たせば`true`である．

## `Drop`とRAII

型に`Drop`トレイトを実装すると，その型の値が片付けられるときに`drop`メソッドが呼ばれる．
値が片付けられるのは，変数がスコープを抜けたとき，関数が`?`により途中で返ったとき，パニックで巻き戻るときである．

```rust
impl Drop for LockFile {
    fn drop(&mut self) {
        if !self.committed {
            let _ = fs::remove_file(&self.lock_path);
        }
    }
}
```

`LockFile`は，作るときに`.lock`のファイルを作り，片付けられるときに(置き換えを終えていなければ)消す．
資源の確保を値の作成に，解放を値の片付けに結び付ける書き方を，RAII(Resource Acquisition Is Initialization)と呼ぶ．
`File`や`TempDir`も同じ仕組みで，ファイルを閉じたりディレクトリを消したりしている．

参照の更新の途中で`?`によってエラーが返っても，`.lock`のファイルは必ず消える．
ほかの言語の`try`と`finally`で書く後片付けを，型が引き受ける．

`let _ = 式;`は，結果を使わないことを明示する．`drop`の中では失敗を返せないので，ファイルを消す失敗は無視する．

## 値を消費するメソッド

`commit`は，`&self`ではなく`self`を受け取る．呼び出すと，`LockFile`の値はメソッドにムーブされる．

```rust
pub fn commit(mut self) -> Result<(), Error> {
    fs::rename(&self.lock_path, &self.path)?;
    self.committed = true;
    Ok(())
}
```

`commit`のあとでロックを使おうとすると，コンパイルエラーになる．
置き換えたあとのロックに書き込むという誤りを，型で防げる．

```text
error[E0382]: borrow of moved value: `lock`
   --> src/repo.rs:247:9
    |
245 |         let mut lock = LockFile::acquire(&self.git_dir.join(name.as_str()))?;
    |             -------- move occurs because `lock` has type `LockFile`, which does not implement the `Copy` trait
246 |         lock.commit()?;
    |              -------- `lock` moved due to this method call
247 |         lock.write_all(format!("{id}\n").as_bytes())
    |         ^^^^ value borrowed here after move
    |
note: `LockFile::commit` takes ownership of the receiver `self`, which moves `lock`
```

`commit`の最後で`self`がスコープを抜けると，`drop`が呼ばれる．`committed`が`true`なので，`drop`は何も消さない．

### RustOwlでムーブを見る

VS Codeで`repo.rs`の`update_ref`を開き，`lock`にカーソルを置く．`lock.commit()`の位置にムーブを表す黄色の下線が現れ，生きている範囲はそこで終わる．
端末では，パッケージのディレクトリで次のように実行する．メソッドは`モジュール::型::メソッド`の形で指定する．

```console
mise run rustowl -- repo::Repository::update_ref lock
```

## ファイルの排他的な作成と置き換え

`OpenOptions`は，ファイルを開く方法を組み合わせて指定する．
`create_new(true)`は，ファイルがなければ作り，すでにあれば`ErrorKind::AlreadyExists`のエラーにする．
「なければ作る」を1回の操作で行うので，2つの処理が同時に作ろうとしても，成功するのは1つだけである．

```rust
let file = OpenOptions::new()
    .write(true)
    .create_new(true)
    .open(&lock_path)?;
```

`fs::rename(from, to)`は，ファイルの名前を変える．`to`がすでにあれば置き換える．
同じファイルシステムの中の名前の変更は，途中の状態が見えない(原子的である)．
読む側は，古い中身か新しい中身のどちらかを読み，書きかけの中身を読むことはない．

パスの後ろに`.lock`を足すには，`path.as_os_str().to_owned()`で`OsString`にしてから`push`する．
`OsString`は，`String`と違い，UTF-8でないファイル名も表せる．

## `Option`の変換

`Option<String>`の`as_deref()`は，`Option<&str>`を返す．
引数を省略できるとき，既定値と合わせて`&str`で扱うのに使う．

```rust
let start: Option<String> = None;
assert_eq!(start.as_deref().unwrap_or("HEAD"), "HEAD");
```

`unwrap_or(既定値)`は，`None`なら既定値を返す．

`match`のパターンには，列挙子のフィールドの値まで書ける．
`Command::Branch { name: None, .. }`は，`name`が`None`の`Branch`だけに一致する．`..`は，残りのフィールドを無視する．
