use crate::{Config, ENVIRON, MyResult, exec_cmd};
use std::process::Command;

/// Killing the previous instances of `wallswitch` before running
pub fn kill_other_instances(config: &Config) -> MyResult<()> {
    let pkg_name = ENVIRON.get_pkg_name();
    let current_pid: u32 = std::process::id();
    let pids: Vec<u32> = get_pids(pkg_name, config)?;

    /*
    pids.into_iter()
        .filter(|&pid| pid != current_pid)
        .try_for_each(|pid| -> MyResult<()> {
            println!("Killing previous instances: kill -9 {pid}\n");
            kill_app(pid)
        })?;
    */

    for pid in pids {
        if pid != current_pid {
            if config.verbose {
                println!("Killing previous instances: kill -9 {pid}\n");
            }
            kill_app(pid, config)?;
        }
    }

    Ok(())
}

fn get_pids(pkg_name: &str, config: &Config) -> MyResult<Vec<u32>> {
    // pgrep -f wallswitch
    let mut cmd = Command::new("pgrep");
    let pgrep_cmd = cmd.arg("-f").arg(pkg_name);
    let pgrep_out = exec_cmd(pgrep_cmd, config.verbose, "pgrep")?;

    // pidof -x wallswitch
    let mut cmd = Command::new("pidof");
    let pidof_cmd = cmd.arg("-x").arg(pkg_name);
    let pidof_out = exec_cmd(pidof_cmd, config.verbose, "pidof")?;

    let pgrep_stdout: Vec<u32> = bytes_to_numbers(&pgrep_out.stdout);
    let pidof_stdout: Vec<u32> = bytes_to_numbers(&pidof_out.stdout);

    let mut pids: Vec<u32> = [pgrep_stdout, pidof_stdout].concat();

    // get unique
    pids.sort();
    pids.dedup();

    if config.verbose {
        println!("Process identification (pid) found:");
        println!("pids: {pids:?}\n");
    }

    Ok(pids)
}

/**
Converts a slice of bytes to a string, including invalid characters.

Then this string is converted into numbers.
*/
fn bytes_to_numbers(bytes: &[u8]) -> Vec<u32> {
    String::from_utf8_lossy(bytes)
        .split(['\n', ' '])
        .flat_map(|s| s.parse().ok())
        .collect()
}

fn kill_app(pid_number: u32, config: &Config) -> MyResult<()> {
    // kill -9 pid_number
    let mut cmd = Command::new("kill");
    let kill = cmd.arg("-9").arg(pid_number.to_string());

    exec_cmd(kill, config.verbose, "kill")?;

    Ok(())
}

#[cfg(test)]
mod test_pids {
    use crate::pids::bytes_to_numbers;

    #[test]
    /// `cargo test -- --show-output pgrep_pids`
    ///
    /// <https://doc.rust-lang.org/reference/expressions/literal-expr.html>
    fn pgrep_pids() {
        let cmd_pgrep = r#"

        123

        abc

        d

        some invalid bytes: \xF0\x90\x80World

        456 xxx 78 44aaa 9

        fgh 


        10
        
        "#;

        println!("cmd_pgrep: '{cmd_pgrep}'\n");

        let bytes: Vec<u8> = cmd_pgrep.bytes().collect();

        println!("bytes: {bytes:?}\n");

        let pgrep_stdout: Vec<u32> = bytes_to_numbers(&bytes);

        println!("pgrep_stdout: {pgrep_stdout:?}");

        assert_eq!(pgrep_stdout, vec![123, 456, 78, 9, 10])
    }
}
