// for macros
fn main() {
    println!("cargo:rerun-if-changed=templates/gradle-maven.init.gradle");
    println!("cargo:rerun-if-changed=templates/intellij-publish.init.gradle");
    println!("cargo:rerun-if-changed=templates/release_notes.init.gradle");
}
