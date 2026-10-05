# テストリスト

## 単体テスト

### revwalk

- [x] 一直線の履歴を，新しいコミットから順に返す
- [x] マージで合流した履歴の共通の祖先を1回だけ返し，コミッターの時刻の新しい順に並べる
- [x] `take(1)`は，最初のコミットだけを返す
- [x] 親のコミットがなければ，エラーを返す

### revision

- [x] `HEAD~0`は`HEAD`，`HEAD~`は最初の親，`main~2`は親の親になる
- [x] ルートを超える`HEAD~3`は，`ObjectNotFound`になる

## 結合テスト

### log

- [x] 3つのコミットの`log`が，`git log --oneline`と一致し，新しい順に並ぶ
- [x] `log -n 1 HEAD~1`が，`git log --oneline -n 1 HEAD~1`と一致する
- [x] マージのある履歴の`log`が，`git log --oneline`と一致し，5つのコミットを1回ずつ表示する
- [x] コミットのない`HEAD`の`log`は，`Not a valid object name HEAD`のエラーになる
