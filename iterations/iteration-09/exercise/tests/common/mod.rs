// 結合テストのファイルごとに使う関数が違うので，使わない関数の警告を出さない．
#![allow(dead_code)]

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// テストで使う作者とコミッター．時刻を決めておくので，コミットのIDが毎回同じになる．
pub fn identity() -> HashMap<String, String> {
    let mut env = HashMap::new();
    for role in ["AUTHOR", "COMMITTER"] {
        env.insert(format!("GIT_{role}_NAME"), String::from("Alice"));
        env.insert(
            format!("GIT_{role}_EMAIL"),
            String::from("alice@example.com"),
        );
        env.insert(
            format!("GIT_{role}_DATE"),
            String::from("@1767225600 +0900"),
        );
    }
    env
}

/// `rgit`を`dir`で実行し，標準出力に書いた文字列を返す．
pub fn rgit(dir: &Path, args: &[&str]) -> String {
    let mut out = Vec::new();
    rgit::cli::run(args, dir, &identity(), &mut out).unwrap();
    String::from_utf8(out).unwrap()
}

/// 作者とコミッターの時刻を`time`(UNIX時刻)にして`rgit`を`dir`で実行し，標準出力を返す．
pub fn rgit_at(dir: &Path, time: i64, args: &[&str]) -> String {
    let mut env = identity();
    for role in ["AUTHOR", "COMMITTER"] {
        env.insert(format!("GIT_{role}_DATE"), format!("@{time} +0900"));
    }
    let mut out = Vec::new();
    rgit::cli::run(args, dir, &env, &mut out).unwrap();
    String::from_utf8(out).unwrap()
}

/// `rgit`を`dir`で実行し，エラーを返す．
pub fn rgit_error(dir: &Path, args: &[&str]) -> rgit::Error {
    let mut out = Vec::new();
    rgit::cli::run(args, dir, &identity(), &mut out).unwrap_err()
}

/// 本物の`git`を`dir`で実行し，標準出力を返す．利用者の設定は読まない．
pub fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .envs(identity())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}
