use arboard::Clipboard;
use std::fs::File;
use std::io::BufReader;

pub use document::Document;
use editor::Editor;
pub use editor::Position;
pub use editor::SearchDirection;
pub use row::Row;
pub use terminal::Terminal;

mod document;
mod editor;
mod highlighting;
mod row;
mod terminal;

mod tokenizer;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let document = if let Some(file_name) = args.get(1) {
        let reader = BufReader::new(File::open(file_name)?);
        Document::new(reader)?
    } else {
        let stdin = std::io::stdin();
        let lines = BufReader::new(stdin.lock());
        Document::new(lines)?
    };

    if document.is_empty() {
        panic!("You must construct additional pylons")
    }

    let clipboard = Clipboard::new().expect("Failed to initialize clipboard");

    let terminal = Terminal::new(None).expect("Failed to initialize terminal");

    match Editor::new(document, Some(clipboard), terminal)
        .expect("Failed to read input.")
        .run()
    {
        Ok(_) => {}
        Err(e) => println!("Error while running program: {e}"),
    }
    Ok(())
}
