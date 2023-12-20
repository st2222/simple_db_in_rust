use std::path::Path;
use simpledb::disk::{BlockId,FileMgr, Page};

fn main() -> std::io::Result<()> {
    // データベースディレクトリとブロックサイズを設定
    let db_directory = Path::new("./temp_db");
    let blocksize = 1024; // 例として1024バイトをブロックサイズとする

    // FileMgrを初期化
    let mut file_mgr = FileMgr::new(&db_directory, blocksize)?;

    // ブロックIDとページを作成
    let blk = BlockId::new("testfile", 0);
    let mut page = Page::new(blocksize);

    // ページにデータをセット
    page.set_string(0, "Hello, world!");
    page.set_int(100, 42);

    // データをファイルに書き込む
    file_mgr.write(&blk, &page)?;

    // 新しいページを作成してデータを読み込む
    let mut new_page = Page::new(blocksize);
    file_mgr.read(&blk, &mut new_page)?;

    // 読み込んだデータを表示
    println!("Read string: {}", new_page.get_string(0));
    println!("Read int: {}", new_page.get_int(100));

    Ok(())
}
