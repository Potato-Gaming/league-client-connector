use league_client_connector::LeagueClientConnector;

#[test]
fn get_path() {
  let result = LeagueClientConnector::get_path().unwrap();

  if cfg!(windows) {
    assert_eq!(result, "C:\\Riot Games\\League of Legends".to_string());
  }

  if cfg!(macos) {
    assert_eq!(result, "");
  }
}

#[test]
fn parse_lockfile() {
  let lockfile = LeagueClientConnector::parse_lockfile().unwrap();

  println!("{:?}", lockfile);

  assert!(lockfile.port > 0);
}
