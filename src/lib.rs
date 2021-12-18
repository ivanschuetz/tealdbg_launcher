use std::{
    error::Error,
    fs,
    io::{self, BufRead, BufReader},
    path::Path,
    process::{ChildStderr, Command, ExitStatus, Stdio},
};

use algonaut::{core::ToMsgPack, transaction::SignedTransaction};

pub struct Config<'a> {
    pub mode: Mode<'a>,
    pub output_files_dir: &'a str,
    pub delete_output_files: bool,
}

impl<'a> Default for Config<'a> {
    fn default() -> Self {
        Config {
            output_files_dir: ".",
            delete_output_files: true,
            mode: Mode::default_sandbox(),
        }
    }
}

pub enum Mode<'a> {
    Sandbox {
        command: &'a str, // `sandbox` if in PATH or `<path>/sandbox`
    },
    /// Private (not sandboxed) network
    Private {
        goal_command: &'a str,     // `goal` if in PATH or `<path>/goal`
        tealdbg_command: &'a str,  // `tealdbg` if in PATH or `<path>/tealdbg`
        node_dir: Option<&'a str>, // node directory, if not using ALGORAND_DATA
    },
}

impl<'a> Mode<'a> {
    pub fn default_sandbox() -> Mode<'a> {
        Mode::Sandbox { command: "sandbox" }
    }

    pub fn default_private() -> Mode<'a> {
        Mode::Private {
            goal_command: "goal",
            tealdbg_command: "tealdbg",
            node_dir: None,
        }
    }
}

pub fn launch_default<T>(
    txns: &[SignedTransaction],
    program_path: &T,
) -> Result<(), Box<dyn Error + Send + Sync>>
where
    T: AsRef<Path> + std::fmt::Debug,
{
    launch(Config::default(), txns, program_path)
}

pub fn launch<T>(
    config: Config,
    txns: &[SignedTransaction],
    program_path: &T,
) -> Result<(), Box<dyn Error + Send + Sync>>
where
    T: AsRef<Path> + std::fmt::Debug,
{
    let mut bytes = vec![];
    for t in txns {
        bytes.push(t.to_msg_pack()?);
    }
    let txs_path = Path::new(config.output_files_dir).join("output.tx");
    fs::write(txs_path.clone(), bytes.concat())?;

    if let Mode::Sandbox { command } = config.mode {
        copy_file_to_sandbox(command, &txs_path)?;
        copy_file_to_sandbox(command, program_path)?;
    }

    println!("Dump status: {:?}", dump(&config.mode)?);

    let tealdbg_stderr =
        tealdbg_stderr(&config.mode, program_path)?.ok_or("Error capturing stderr")?;
    BufReader::new(tealdbg_stderr)
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| println!("{}", line));

    if config.delete_output_files {
        fs::remove_file(txs_path)?;
        if let Mode::Private { .. } = config.mode {
            let dump_file = Path::new(config.output_files_dir).join("dr.msgp");
            fs::remove_file(dump_file)?;
        }
    }

    Ok(())
}

fn copy_file_to_sandbox<T>(
    sandbox_command: &str,
    path: &T,
) -> Result<(), Box<dyn Error + Send + Sync>>
where
    T: AsRef<Path>,
{
    let mut copy = Command::new(sandbox_command);
    let copy_command = copy
        .arg("copyTo")
        .arg(path.as_ref())
        .stderr(Stdio::piped())
        .spawn()?
        .stderr
        .ok_or("Error capturing stderr")?;
    BufReader::new(copy_command)
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| println!("{}", line));
    Ok(())
}

fn dump(mode: &Mode) -> io::Result<ExitStatus> {
    let mut cmd = match mode {
        Mode::Sandbox { command } => Command::new(command),
        Mode::Private {
            goal_command: command,
            ..
        } => Command::new(command),
    };
    let prefix = match mode {
        Mode::Sandbox { .. } => cmd.arg("goal"),
        Mode::Private {
            goal_command: _, ..
        } => &mut cmd,
    };

    prefix
        .arg("clerk")
        .arg("dryrun")
        .arg("-t")
        .arg("output.tx")
        .arg("--dryrun-dump")
        .arg("-o")
        .arg("dr.msgp")
        .status()
}

fn tealdbg_stderr<T>(
    mode: &Mode,
    program_path: &T,
) -> Result<Option<ChildStderr>, Box<dyn Error + Send + Sync>>
where
    T: AsRef<Path> + std::fmt::Debug,
{
    let mut cmd = match mode {
        Mode::Sandbox { command } => Command::new(command),
        Mode::Private {
            goal_command: command,
            ..
        } => Command::new(command),
    };
    let prefix = match mode {
        Mode::Sandbox { .. } => cmd.arg("tealdbg"),
        Mode::Private { .. } => &mut cmd,
    };

    let file_name = program_path.as_ref().file_name().ok_or_else(|| {
        format!(
            "Couldn't get filename from program path: {:?}",
            program_path
        )
    })?;

    Ok(prefix
        .arg("debug")
        .arg(file_name)
        .arg("-d")
        .arg("dr.msgp")
        .stderr(Stdio::piped())
        .spawn()?
        .stderr)
}
