use super::*;

#[test]
fn templates_are_in_the_data_folder_unless_the_variable_names_one() {
    let data = PathBuf::from("data");
    assert_eq!(templates_dir_from(None, Some(data.clone())), Some(data.join("templates")));
    assert_eq!(templates_dir_from(Some("ours".into()), Some(data)), Some(PathBuf::from("ours")));
    assert_eq!(templates_dir_from(None, None), None);
    assert!(data_dir("OpenECraft").is_none_or(|d| d.ends_with("OpenECraft/data")));
}
