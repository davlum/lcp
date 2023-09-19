mod document;
mod editor;
mod highlighting;
mod row;
mod terminal;
mod tokenizer;

pub use document::Document;
use editor::Editor;
pub use editor::Position;
pub use editor::SearchDirection;
pub use row::Row;
pub use terminal::Terminal;

fn main() {
    match Editor::default().expect("Failed to read input.").run() {
        Ok(_) => {}
        Err(e) => println!("Error while running program: {e}"),
    }
}
