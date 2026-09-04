use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use pdf_process::html_to_typst::html_to_typst;

#[test]
#[ignore = "requires the optional typst CLI"]
fn generated_typst_compiles() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "pdf-process-typst-test-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).unwrap();
    let source_path = directory.join("document.typ");
    let output_path = directory.join("document.pdf");
    fs::write(
        &source_path,
        html_to_typst("<h1>Title</h1><p>Area</p><p><span class='math'>A = pi r^2</span></p>"),
    )
    .unwrap();

    let status = Command::new("typst")
        .args(["compile"])
        .arg(&source_path)
        .arg(&output_path)
        .status()
        .unwrap();
    assert!(status.success());
    assert!(output_path.is_file());
    fs::remove_dir_all(directory).unwrap();
}
