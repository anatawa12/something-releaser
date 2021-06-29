use std::fs::{create_dir_all, read_to_string, File};
use std::io::Write;
use std::path::Path;

// for macros
fn main() {
    create("templates/gradle-maven.init.gradle");
    create("templates/intellij-publish.init.gradle");
    create("templates/release_notes.init.gradle");
}

fn create(path: &str) {
    println!("cargo:rerun-if-changed={}", path);
    let source = read_to_string(path).unwrap();

    let pat = source
        .replace("{", "{{")
        .replace("}", "}}")
        .replace("<<", "{")
        .replace(">>", "}");

    let path = Path::new(&std::env::var("OUT_DIR").unwrap()).join(path);
    create_dir_all(path.parent().unwrap()).unwrap();
    File::create(path)
        .unwrap()
        .write_all(pat.as_bytes())
        .unwrap();
}
