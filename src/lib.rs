use std::{
    error::Error,
    fs,
    io::{BufRead, BufReader},
    path::Path,
    process::{Command, Stdio},
};

use algonaut::{core::ToMsgPack, transaction::SignedTransaction};

pub struct Config<'a> {
    pub goal_command: &'a str, // override with `sandbox goal` or `<path>/goal`
    pub output_files_dir: &'a str,
    pub node_dir: Option<&'a str>, // node directory, if not using ALGORAND_DATA
    pub tealdbg_command: &'a str,
}

impl<'a> Default for Config<'a> {
    fn default() -> Self {
        Config {
            goal_command: "goal",
            output_files_dir: ".",
            node_dir: None,
            tealdbg_command: "tealdbg",
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
    fs::write("./output.tx", bytes.concat())?;

    let dump_file = Path::new(config.output_files_dir).join("dr.msgp");

    let mut dump = Command::new(config.goal_command);
    let dump_command = dump
        .arg("clerk")
        .arg("dryrun")
        .arg("-t")
        .arg(Path::new(config.output_files_dir).join("output.tx"))
        .arg("--dryrun-dump")
        .arg("-o")
        .arg(dump_file.clone());

    if let Some(node_dir) = config.node_dir {
        dump_command.arg("-d").arg(node_dir);
    }

    let dump_exit_status = dump_command
        .status()
        .expect("context file generation failed");

    println!("dump status: {:?}", dump_exit_status);

    let tealdbg_stdout = Command::new(config.tealdbg_command)
        .arg("debug")
        .arg(program_path.as_ref())
        .arg("-d")
        .arg(dump_file)
        .stderr(Stdio::piped())
        .spawn()?
        .stderr
        .ok_or_else(|| "Error capturing stderr")?;

    BufReader::new(tealdbg_stdout)
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| println!("{}", line));

    Ok(())
}
