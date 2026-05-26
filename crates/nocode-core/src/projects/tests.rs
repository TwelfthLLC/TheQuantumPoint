use super::*;
use std::path::Path;

#[test]
fn appends_project_folder_under_parent() {
    let parent = Path::new("C:/Users/Me/Documents");
    let got = resolve_project_directory(parent, "Yangi loyiha");
    assert_eq!(got, parent.join("Yangi loyiha"));
}

#[test]
fn keeps_path_when_already_named() {
    let root = Path::new("C:/Users/Me/Documents/Yangi loyiha");
    let got = resolve_project_directory(root, "Yangi loyiha");
    assert_eq!(got, root);
}
