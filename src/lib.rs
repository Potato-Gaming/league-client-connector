use base64::encode;
use regex::Regex;
use std::env::consts::OS;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::result::Result;

pub struct LeagueClientConnector {}

impl LeagueClientConnector {
    pub fn get_path() -> Result<String, ()> {
        let output: String = match OS {
            "windows" => {
                let output_child = Command::new("WMIC")
                    .args(&[
                        "PROCESS",
                        "WHERE",
                        "name='LeagueClientUx.exe'",
                        "GET",
                        "commandline",
                    ])
                    .output()
                    .expect("Failed to run WMIC");

                let res = String::from_utf8(output_child.stdout).unwrap();
                let pattern =
                    Regex::new(r"--install-directory=(?P<dir>[[:alnum:][:space:]:\.\\]+)").unwrap();
                let caps = pattern.captures(&res).unwrap();
                caps["dir"].to_string()
            }
            "macos" => {
                // https://rust-lang-nursery.github.io/rust-cookbook/os/external.html#run-piped-external-commands
                let mut ps_output_child = Command::new("ps")
                    .args(&["x", "-o", "args"])
                    .stdout(Stdio::piped())
                    .spawn()
                    .unwrap();

                if let Some(ps_output) = ps_output_child.stdout.take() {
                    let output_child = Command::new("grep")
                        .args(&["LeagueClientUx"])
                        .stdin(ps_output)
                        .stdout(Stdio::piped())
                        .spawn()
                        .unwrap();

                    let output = output_child.wait_with_output().unwrap();
                    ps_output_child.wait().unwrap();
                    let res = String::from_utf8(output.stdout).unwrap();

                    let pattern =
                        Regex::new(r"--install-directory=(?P<dir>[[:alnum:][:space:]:\./\\]+)")
                            .unwrap();
                    let caps = pattern.captures(&res).unwrap();
                    caps["dir"].to_string().trim().to_string()
                } else {
                    panic!("Unable to get ps results");
                }
            }
            _ => unimplemented!(),
        };

        println!("output {:?}", output);

        Ok(output)
    }

    pub fn parse_lockfile() -> Result<RiotLockFile, ()> {
        let mut path = PathBuf::from(Self::get_path().unwrap());
        path.push("lockfile");
        let lockfile = path.to_str().unwrap();

        let contents = fs::read_to_string(lockfile).expect("Failed to read lockfile");

        let pieces: Vec<&str> = contents.split(":").collect();

        let username = "riot".to_string();
        let address = "127.0.0.1".to_string();
        let password = pieces[3].to_string();
        let b64 = encode(format!("{}:{}", username, password).as_bytes());

        Ok(RiotLockFile {
            process: pieces[0].to_string(),
            pid: pieces[1].parse().unwrap(),
            port: pieces[2].parse().unwrap(),
            password,
            protocol: pieces[4].to_string(),
            username,
            address,
            b64,
        })
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
    pub b64: String,
}
