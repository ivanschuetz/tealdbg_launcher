use std::{
    error::Error,
    fs,
    io::{BufRead, BufReader},
    path::Path,
    process::{Command, Stdio},
};

use algonaut::{core::ToMsgPack, transaction::SignedTransaction};

pub struct Config<'a> {
    pub goal_command: &'a str,     // override with `<path>/goal` if not in path
    pub tealdbg_command: &'a str,  // override with `<path>/tealdbg` if not in path
    pub node_dir: Option<&'a str>, // node directory, if not using ALGORAND_DATA
    pub output_files_dir: &'a str,
    pub delete_output_files: bool,
}

impl<'a> Default for Config<'a> {
    fn default() -> Self {
        Config {
            goal_command: "goal",
            tealdbg_command: "tealdbg",
            node_dir: None,
            output_files_dir: ".",
            delete_output_files: true,
        }
    }
}

pub fn launch_default<T: AsRef<Path>>(
    txns: &[SignedTransaction],
    program_path: T,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    launch(Config::default(), txns, program_path)
}

pub fn launch<T: AsRef<Path>>(
    config: Config,
    txns: &[SignedTransaction],
    program_path: T,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut bytes = vec![];
    for t in txns {
        bytes.push(t.to_msg_pack()?);
    }
    let txns_file = Path::new(config.output_files_dir).join("output.tx");
    fs::write(txns_file.clone(), bytes.concat())?;

    let dump_file = Path::new(config.output_files_dir).join("dr.msgp");

    let mut dump = Command::new(config.goal_command);
    let dump_command = dump
        .arg("clerk")
        .arg("dryrun")
        .arg("-t")
        .arg(txns_file.clone())
        .arg("--dryrun-dump")
        .arg("-o")
        .arg(dump_file.clone());

    if let Some(node_dir) = config.node_dir {
        dump_command.arg("-d").arg(node_dir);
    }

    println!("dump status: {:?}", dump_command.status()?);

    let tealdbg_stderr = Command::new(config.tealdbg_command)
        .arg("debug")
        .arg(program_path.as_ref())
        .arg("-d")
        .arg(dump_file.clone())
        .stderr(Stdio::piped())
        .spawn()?
        .stderr
        .ok_or("Error capturing stderr")?;

    BufReader::new(tealdbg_stderr)
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| println!("{}", line));

    if config.delete_output_files {
        fs::remove_file(txns_file)?;
        fs::remove_file(dump_file)?;
    }

    Ok(())
}
