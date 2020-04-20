use mdnotes::MdNotesRuntime;

fn main() {
    let runtime = MdNotesRuntime::new().unwrap();

    let home_dir = dirs::home_dir().unwrap();
    let id = runtime.open_notes(home_dir.join("code/notes"));

    println!("http://localhost:{}/{}/static/", runtime.server_port(), id);

    // loop {}

    runtime.close_notes(id);
}
