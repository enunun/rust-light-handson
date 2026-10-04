# テストリスト

## 単体テスト

### object

- [x] 空のバイト列は，空の文字列になる(`to_hex`)
- [x] 各バイトは，2桁の小文字の16進数になる(`[0x00, 0x0f, 0xab, 0xff]`は`000fabff`，`to_hex`)
- [x] 空のデータのblobのハッシュは`e69de29bb2d1d6434b8b29ae775ad8c2e48c5391`になる
- [x] `hello\n`のblobのハッシュは`ce013625030ba8dba906f756967f9e9ca394464a`になる

## 結合テスト

### hash_blob

- [x] `hello\n`のハッシュが，`git hash-object --stdin`の結果と一致する
- [x] 文字列でないバイト列`[0x00, 0xff, 0x10]`のハッシュが，`git hash-object --stdin`の結果と一致する
