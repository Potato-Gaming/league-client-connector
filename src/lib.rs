//! # league_client_connector
//!
//! Rust implementation for [lcu-connector](https://github.com/Pupix/lcu-connector) minus the
//! file watching mechanism. This crate needs the League Client to be opened, in order to get the
//! installation path for League of Legends so the `lockfile` can be retrieved correctly.
//!
//! Note that every time the League Client is opened, it creates a new `lockfile` so a watcher or
//! some refresh mechanism needs to be implemented to use correctly in an application.
//!
//! The contents of the `lockfile` are parsed and presented in a readable format so a connection to
//! the [Game Client API](https://developer.riotgames.com/docs/lol#game-client-api) can be
//! established.

use base64::encode;
use regex::Regex;
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use std::env::consts::OS;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Make sure the League of Legends Client is opened before running any of the methods.
pub struct LeagueClientConnector {}

impl LeagueClientConnector {
    /// Parses League's client file which contains information needed to connect to
    /// [Game Client API](https://developer.riotgames.com/docs/lol#game-client-api)
    /// Which uses RESTful to interact with League's Client
    ///
    /// # Example
    ///
    /// ```
    /// use league_client_connector::LeagueClientConnector;
    ///
    /// let lockfile = LeagueClientConnector::parse_lockfile().unwrap();
    ///
    /// println!("{:?}", lockfile);
    ///
    /// assert!(lockfile.port > 0);
    /// ```
    pub fn parse_lockfile() -> Result<RiotLockFile> {
        let mut path = PathBuf::from(Self::get_path()?);
        path.push("lockfile");

        let lockfile = path.to_str().ok_or(LeagueConnectorError::EmptyPath {})?;

        let contents = fs::read_to_string(lockfile).context(UnableToRead)?;

        let pieces: Vec<&str> = contents.split(":").collect();

        let username = "riot".to_string();
        let address = "127.0.0.1".to_string();
        let process = pieces[0].to_string();
        let pid = pieces[1].parse().context(NumberParse { name: "pid" })?;
        let port = pieces[2].parse().context(NumberParse { name: "port" })?;
        let password = pieces[3].to_string();
        let protocol = pieces[4].to_string();
        let b64_auth = encode(format!("{}:{}", username, password).as_bytes());

        Ok(RiotLockFile {
            process,
            pid,
            port,
            password,
            protocol,
            username,
            address,
            b64_auth,
        })
    }

    /// Gets League of Legends Installation path. Useful to find the "lockfile" for example.
    /// Works for Windows & Mac OSX
    ///
    /// # Example
    ///
    /// ```
    /// use league_client_connector::LeagueClientConnector;
    ///
    /// let path = LeagueClientConnector::get_path().unwrap();
    ///
    /// assert!(path.len() > 0);
    /// ```
    pub fn get_path() -> Result<String> {
        let raw_info: String = match OS {
            "windows" => Self::get_raw_league_info_in_windows()?,
            "macos" => Self::get_raw_league_info_in_macos()?,
            _ => unimplemented!(),
        };

        let pattern = Regex::new(r"--install-directory=(?P<dir>[[:alnum:][:space:]:\./\\]+)")
            .context(RegexParse)?;

        let caps = pattern
            .captures(&raw_info)
            .ok_or(LeagueConnectorError::NoInstallationPath {})?;

        let path = caps["dir"].to_string().trim().to_string();

        Ok(path)
    }

    fn get_raw_league_info_in_windows() -> Result<String> {
        let output_child = Command::new("WMIC")
            .args(&[
                "PROCESS",
                "WHERE",
                "name='LeagueClientUx.exe'",
                "GET",
                "commandline",
            ])
            .output()
            .context(GetRawPath)?;

        let res = String::from_utf8(output_child.stdout).context(Utf8Parse)?;

        Ok(res)
    }

    fn get_raw_league_info_in_macos() -> Result<String> {
        let mut ps_output_child = Command::new("ps")
            .args(&["x", "-o", "args"])
            .stdout(Stdio::piped())
            .spawn()
            .context(GetRawPath)?;

        let ps_output = if let Some(ps_output) = ps_output_child.stdout.take() {
            ps_output
        } else {
            return Err(LeagueConnectorError::EmptyStdout {});
        };

        let output_child = Command::new("grep")
            .args(&["LeagueClientUx"])
            .stdin(ps_output)
            .stdout(Stdio::piped())
            .spawn()
            .context(GetRawPath)?;

        let output = output_child.wait_with_output().context(GetRawPath)?;
        ps_output_child.wait().context(GetRawPath)?;
        let res = String::from_utf8(output.stdout).context(Utf8Parse)?;

        Ok(res)
    }
}

/// This struct can be used to establish a connection with
/// [Game Client API](https://developer.riotgames.com/docs/lol#game-client-api) like so
///
/// ```bash
/// curl --request GET \
/// --url https://127.0.0.1:54835/lol-summoner/v1/current-summoner \
/// --header 'authorization: Basic cmlvdDpDMERXVDZWREoySDUwSEZKMkJFU2hR'
/// ```
///
/// Note that all the information is gotten from the lockfile:
/// - protocol: https
/// - address: 127.0.0.1
/// - b64_auth: cmlvdDpDMERXVDZWREoySDUwSEZKMkJFU2hR
///
/// For the actual endpoint, download the [Rift Explorer](https://github.com/Pupix/rift-explorer)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RiotLockFile {
    pub process: String,
    pub pid: u32,
    pub port: u32,
    pub password: String,
    pub protocol: String,
    pub username: String,
    pub address: String,
    pub b64_auth: String,
}

pub type Result<T, E = LeagueConnectorError> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
pub enum LeagueConnectorError {
    #[snafu(display("Could not get raw path: {}", source))]
    GetRawPath { source: std::io::Error },

    #[snafu(display("Process didn't return any stdout"))]
    EmptyStdout {},

    #[snafu(display("Unable to parse from utf8: {}", source))]
    Utf8Parse { source: std::string::FromUtf8Error },

    #[snafu(display("Unable to parse Regex: {}", source))]
    RegexParse { source: regex::Error },

    #[snafu(display("No installation path found for League"))]
    NoInstallationPath {},

    #[snafu(display("Path is empty"))]
    EmptyPath {},

    #[snafu(display("Unable to read file: {}", source))]
    UnableToRead { source: std::io::Error },

    #[snafu(display("Unable to parse to number {}: {}", name, source))]
    NumberParse {
        source: std::num::ParseIntError,
        name: &'static str,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lockfile_is_equal() {
        let file1 = build_lockfile(1337, "127.0.0.1", "some_b64");
        let file2 = build_lockfile(1337, "127.0.0.1", "some_b64");

        assert_eq!(file1, file2);
    }

    #[test]
    fn lockfile_diff_port() {
        let file1 = build_lockfile(1337, "127.0.0.1", "some_b64");
        let file2 = build_lockfile(1338, "127.0.0.1", "some_b64");

        assert_ne!(file1, file2);
    }

    #[test]
    fn lockfile_diff_address() {
        let file1 = build_lockfile(1337, "127.0.0.1", "some_b64");
        let file2 = build_lockfile(1337, "127.0.0.2", "some_b64");

        assert_ne!(file1, file2);
    }

    #[test]
    fn lockfile_diff_auth() {
        let file1 = build_lockfile(1337, "127.0.0.1", "some_b64");
        let file2 = build_lockfile(1337, "127.0.0.1", "another_b64");

        assert_ne!(file1, file2);
    }

    fn build_lockfile(port: u32, address: &str, b64_auth: &str) -> RiotLockFile {
        RiotLockFile {
            process: "1234".to_string(),
            pid: 1234,
            port,
            password: "some_password".to_string(),
            protocol: "https".to_string(),
            username: "some_username".to_string(),
            address: address.to_string(),
            b64_auth: b64_auth.to_string(),
        }
    }
}
