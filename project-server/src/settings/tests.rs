use super::*;

fn args(cli: &[&str]) -> Args {
    Args::try_parse_from(std::iter::once("openecraft-project-server").chain(cli.iter().copied())).unwrap()
}

fn file(text: &str) -> File {
    server_config::parse(text).unwrap()
}

#[test]
fn without_options_the_defaults_apply() {
    let s = Settings::merge(args(&["--token", "", "--admin-user", " "]), File::default());
    assert_eq!(s.bind, SocketAddr::from(([127, 0, 0, 1], DEFAULT_PORT)));
    assert_eq!(s.data, PathBuf::from("projects"));
    assert_eq!(s.access, PathBuf::from("projects").join("access.json"));
    assert_eq!((s.token, s.admin_user), (None, None), "empty values are no values");
}

#[test]
fn the_file_sets_what_the_command_line_leaves_open() {
    let f = file("bind = \"0.0.0.0:9001\"\ndata = \"/srv/projects\"\ntoken = \"secret\"\nadmin_user = \"admin\"\n");
    let s = Settings::merge(args(&["--token", "other"]), f);
    assert_eq!(s.bind, "0.0.0.0:9001".parse().unwrap());
    assert_eq!(s.data, PathBuf::from("/srv/projects"));
    assert_eq!(s.access, PathBuf::from("/srv/projects").join("access.json"));
    assert_eq!(s.token.as_deref(), Some("other"), "the command line wins");
    assert_eq!(s.admin_user.as_deref(), Some("admin"));
}

#[test]
fn paths_in_the_file_are_relative_to_it() {
    let dir = std::env::temp_dir().join(format!("openecraft-project-settings-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let config = dir.join("project-server.toml");
    std::fs::write(&config, "data = \"projects\"\naccess = \"etc/access.json\"\n").unwrap();
    let s = Settings::new(args(&["--config", config.to_str().unwrap()])).unwrap();
    assert_eq!(s.data, dir.join("projects"));
    assert_eq!(s.access, dir.join("etc/access.json"));
    std::fs::write(&config, "port = 8091\n").unwrap();
    assert!(Settings::new(args(&["--config", config.to_str().unwrap()])).unwrap_err().contains("port"), "unknown keys are refused");
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn the_example_configuration_is_valid() {
    let f = file(include_str!("../../project-server.example.toml"));
    assert!(f.bind.is_some() && f.data.is_some());
}

#[test]
fn the_admin_user_can_come_from_the_environment() {
    use clap::CommandFactory;
    let command = Args::command();
    let arg = command.get_arguments().find(|a| a.get_id() == "admin_user").unwrap();
    assert_eq!(arg.get_env(), Some(std::ffi::OsStr::new("OPENECRAFT_PROJECT_ADMIN_USER")));
    let s = Settings::merge(args(&["--admin-user", " "]), File::default());
    assert_eq!(s.admin_user, None, "an empty name is no administrator");
}
