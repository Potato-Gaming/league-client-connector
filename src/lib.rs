use base64::encode;
use regex::Regex;
use snafu::{ResultExt, Snafu};
use std::env::consts::OS;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub struct LeagueClientConnector {}

impl LeagueClientConnector {
    /// Parses League's client file which contains information needed to connect to
    /// [Game Client API](https://developer.riotgames.com/docs/lol#game-client-api)
    /// Which uses RESTful to interact with League's Client
    pub fn parse_lockfile() -> Result<RiotLockFile> {
        let mut path = PathBuf::from(Self::get_path()?);
        path.push("lockfile");
        let lockfile = match path.to_str() {
            Some(l) => l,
            None => {
                return Err(LeagueConnectorError::EmptyPath {});
            }
        };

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
    pub fn get_path() -> Result<String> {
        let raw_info: String = match OS {
            "windows" => Self::get_raw_league_info_in_windows()?,
            "macos" => Self::get_raw_league_info_in_macos()?,
            _ => unimplemented!(),
        };

        let pattern = Regex::new(r"--install-directory=(?P<dir>[[:alnum:][:space:]:\./\\]+)")
            .context(RegexParse)?;
        let caps = match pattern.captures(&raw_info) {
            Some(c) => c,
            None => {
                return Err(LeagueConnectorError::NoInstallationPath {});
            }
        };
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

#[derive(Debug)]
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
