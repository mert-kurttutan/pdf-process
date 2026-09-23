use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use pdf_process::mathjax_preview::{render_mathjax_preview, write_mathjax_preview};

#[test]
fn mathjax_preview_defaults_to_a_sidecar() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory =
        std::env::temp_dir().join(format!("pdf-process-test-{}-{unique}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let html_path = directory.join("paper.html");
    fs::write(&html_path, "<p><math>y = x</math></p>").unwrap();
    let output = write_mathjax_preview(&html_path, None).unwrap();
    assert_eq!(output, directory.join("paper.mathjax.html"));
    assert!(fs::read_to_string(output).unwrap().contains("mathjax@3"));
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn mathjax_preview_does_not_duplicate_its_script() {
    let preview = render_mathjax_preview("<head></head><body>ok</body>");
    assert_eq!(render_mathjax_preview(&preview), preview);
}
