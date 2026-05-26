mod document;
mod editor;
mod history;
mod io;
mod tools;

use tiles::{run, Config};
use editor::Editor;

fn main() {
    let config = Config::builder()
        .title("Tiles Editor")
        .width(1200)
        .height(900)
        .viewport(384.0, 256.0)
        .no_file()
        .build();

    run(Editor::new(), config).unwrap();
}
