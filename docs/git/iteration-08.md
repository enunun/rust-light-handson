# Iteration 8：コミットのグラフと`log`

Iteration 8では，コミットの親をたどって履歴を表示する`log`と，`HEAD~1`のようなリビジョンの指定を作る．
このノートでは，コミットのグラフの形と，`git log`がコミットを並べる順序を説明する．

## コミットのグラフ

各コミットは親のIDを持つ．子から親へ矢印を引くと，コミットはグラフになる．
親は子より先に作られるので，矢印をたどって元のコミットに戻ることはない．このようなグラフを有向非巡回グラフ(DAG)と呼ぶ．

```console
$ git log --oneline --graph
*   5d31e22 merge
|\  
| * fa41f39 side
* | 511cd4f third
* | 53b8b96 second
|/  
* a838686 first
```

- `first`は親のないルートのコミットである．
- `second`と`side`は，どちらも`first`を親に持つ．履歴はここで枝分かれしている．
- `merge`は2つの親(`third`と`side`)を持つマージのコミットである．

```console
$ git cat-file -p HEAD | head -4
tree 0d8fc565bd021dba609db929585fd4216ffa7b6b
parent 511cd4f976b9589bed9356df7cfc2a9e4080f173
parent fa41f396bdec0fda49fc86ecf7cf3f50dce3a594
author Alice <alice@example.com> 1767250000 +0900
```

最初の`parent`を最初の親と呼ぶ．`git merge`では，マージした時点で`HEAD`にあったコミットが最初の親になる．

ブランチは，グラフの中の1つのコミットを指す名前にすぎない．
コミットを作ると，新しいコミットが前のコミットを親に持ち，ブランチは新しいコミットを指すように進む．

## `git log`の順序

`git log`は，指定したコミットから親をたどり，たどり着けるコミットをすべて表示する．

```console
$ git log --oneline
5d31e22 merge
fa41f39 side
511cd4f third
53b8b96 second
a838686 first
```

マージがあると，1つのコミットから2つの親へ進むので，次に表示するコミットの候補が複数になる．
`git log`は，候補の中からコミッターの時刻が最も新しいものを選んで表示し，その親を候補に加える，という手順をくり返す．
候補を時刻の順に取り出すには，優先度付きの待ち行列(ヒープ)を使う．

`first`には`second`と`side`の両方からたどり着くが，表示は1回だけである．
すでに候補に加えたコミットを覚えておき，2回目は加えない．

## `~N`による指定

`<rev>~N`は，`<rev>`から最初の親をN回たどったコミットを指す．`~`だけなら`~1`と同じである．

```console
$ git rev-parse --short HEAD~1
511cd4f
$ git rev-parse --short HEAD~3
a838686
$ git log --oneline -n 2 HEAD~1
511cd4f third
53b8b96 second
```

`HEAD~1`は最初の親の`third`である．2番目の親の`side`は，`HEAD^2`で指定する(`rgit`は扱わない)．

ルートのコミットから先に親はないので，`HEAD~4`のように親の数を超える指定はエラーになる．
