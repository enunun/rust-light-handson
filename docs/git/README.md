# Gitのノート

各Iterationで初めて扱うGitの仕組みを説明する．本物の`git`のコマンドで`.git`の中を観察しながら読む．

| Iteration | 内容 |
| --- | --- |
| [0](iteration-00.md) | 内容アドレス，blobオブジェクトとヘッダー，SHA-1 |
| [1](iteration-01.md) | 20バイトと40桁の表記，短縮形 |
| [2](iteration-02.md) | `.git`の構成，`HEAD`，ゆるいオブジェクト，zlib |
| [3](iteration-03.md) | オブジェクトを読む手順，`git cat-file`，短縮形の解決と曖昧さ |
| [4](iteration-04.md) | treeオブジェクト，`git ls-tree`，ファイルのモード，treeの内容のバイト列 |
| [5](iteration-05.md) | インデックスの役割，ファイルの状態，インデックスの形式 |
| [6](iteration-06.md) | インデックスからtreeを作る手順，エントリーの並び順，commitオブジェクト，署名と時刻，作者とコミッターの環境変数 |
| [7](iteration-07.md) | 参照，参照名の規則，`HEAD`とシンボリック参照，`git commit`の手順，ロックファイル |
| [8](iteration-08.md) | コミットのグラフ，マージ，`git log`の順序，`~N`による指定 |
| [9](iteration-09.md) | HEAD，インデックス，作業ディレクトリ，`git status --porcelain`の形，ファイルの状態による省略 |
| [10](iteration-10.md) | `git diff`と`--cached`，最短の編集，編集グラフとMyersのアルゴリズム，unified形式，ハンクと文脈，追加，削除，モードの変更，末尾の改行，バイナリーファイル |
