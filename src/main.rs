use std::env;
use std::fs;
use std::path::Path;
use std::process;

use chrono::Local;

/// 時刻プレフィックスのフォーマット (yyyyMMdd_HHmmss_)
const PREFIX_LEN: usize = 16;

/// エントリポイント。ファイル一覧を取得し、時刻プレフィックスの有無に応じて
/// 付与または除去を一括実行する。
fn main() {
    let files = parse_args();
    let should_strip = has_any_prefixed_file(&files);

    if should_strip {
        for file in &files {
            strip_timestamp_prefix(file);
        }
    } else {
        // 付与する場合はタイムスタンプを1回だけ取得し、全ファイルに同じ日時を付ける。
        let ts = timestamp();
        for file in &files {
            add_timestamp_prefix(file, &ts);
        }
    }
}

/// コマンドライン引数からファイル一覧を取得する。
/// 引数が1つもない場合は usage を表示して終了する。
fn parse_args() -> Vec<String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: rename_time_toggle <file...>");
        process::exit(1);
    }
    args
}

/// 指定されたファイルのいずれかのファイル名が
/// "yyyyMMdd_HHmmss_" 形式のプレフィックスを持つかを判定する。
fn has_any_prefixed_file(files: &[String]) -> bool {
    files.iter().any(|file| {
        Path::new(file)
            .file_name()
            .map(|name| has_timestamp_prefix(&name.to_string_lossy()))
            .unwrap_or(false)
    })
}

/// ファイル名の先頭が "yyyyMMdd_HHmmss_" 形式かを判定する。
/// 判定は文字列を直接走査し、正規表現を使わない。
fn has_timestamp_prefix(name: &str) -> bool {
    let bytes = name.as_bytes();
    bytes.len() > PREFIX_LEN
        && bytes[..8].iter().all(|b| b.is_ascii_digit())
        && bytes[8] == b'_'
        && bytes[9..15].iter().all(|b| b.is_ascii_digit())
        && bytes[15] == b'_'
}

/// ファイル名の先頭から "yyyyMMdd_HHmmss_" を取り除いてリネームする。
fn strip_timestamp_prefix(file: &str) {
    let (path, name) = split_path_and_name(file);
    let new_name = name[PREFIX_LEN..].to_string();
    rename_file(&path, name, &new_name);
}

/// ファイル名の先頭に指定されたタイムスタンプ "yyyyMMdd_HHmmss_" を付けてリネームする。
fn add_timestamp_prefix(file: &str, ts: &str) {
    let (path, name) = split_path_and_name(file);
    let new_name = format!("{}{}", ts, name);
    rename_file(&path, name, &new_name);
}

/// 実行時のシステム日時を "yyyyMMdd_HHmmss" 形式の文字列で返す。
fn timestamp() -> String {
    Local::now().format("%Y%m%d_%H%M%S").to_string()
}

/// パスから親ディレクトリ付きのパスとファイル名に分解する。
/// ファイル名を取得できない場合はエラーを出力して None 相当で返す。
fn split_path_and_name(file: &str) -> (&Path, &str) {
    let path = Path::new(file);
    match path.file_name().and_then(|n| n.to_str()) {
        Some(name) => (path, name),
        None => {
            eprintln!("skip {}: no file name", file);
            process::exit(1);
        }
    }
}

/// ファイルを新しい名前でリネームする。結果を標準出力または標準エラーに出力する。
fn rename_file(path: &Path, old_name: &str, new_name: &str) {
    let new_path = path.with_file_name(new_name);
    match fs::rename(path, &new_path) {
        Ok(()) => println!("{} -> {}", old_name, new_name),
        Err(e) => eprintln!("failed to rename {}: {}", old_name, e),
    }
}
