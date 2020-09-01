use league_client_connector::LeagueClientConnector;
use std::env::consts::OS;

#[test]
fn get_path() {
  let result = LeagueClientConnector::get_path().unwrap();

  match OS {
    "windows" => {
      assert_eq!(result, "C:\\Riot Games\\League of Legends".to_string());
    }
    "macos" => {
      assert_eq!(
        result,
        "/Applications/League of Legends.app/Contents/LoL".to_string()
      );
    }
    _ => unimplemented!(),
  }
}

#[test]
fn parse_lockfile() {
  let lockfile = LeagueClientConnector::parse_lockfile().unwrap();

  println!("{:?}", lockfile);

  assert!(lockfile.port > 0);
}

#[test]
fn equality() {
  let lockfile1 = LeagueClientConnector::parse_lockfile().unwrap();
  let lockfile2 = LeagueClientConnector::parse_lockfile().unwrap();

  assert!(lockfile1 == lockfile2, true);
}
