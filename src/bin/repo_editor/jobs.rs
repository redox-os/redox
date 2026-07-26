// Contains code related to background tasks

use std::fmt::{Display as FmtDisplay, Write as FmtWrite};
use std::io::Read;
use std::sync::mpsc;
use std::thread;
use std::{collections::HashMap, io::Write};

use cookbook::cook::pty::{setup_pty_with_stdin, spawn_to_pipe};
use cookbook::cook::tui::drain_buffer_to_lines;
use pkg::PackageName;

pub type JobId = u64;

pub enum StatusUpdate {
    PushLog(JobId, Vec<u8>),
    JobFinished(JobId, i32),
}

pub struct PackageJob {
    pub id: JobId,
    pub command: String,
    pub targets: Vec<PackageName>,
    pub logs: Vec<String>,
    pub buffer: Vec<u8>,
    pub scroll: usize,
    pub auto_scroll: bool,
    pub exit_code: Option<i32>,
    pub pid: Option<u32>,
    pub pty_writer: Box<dyn Write + Send>,
}

impl PackageJob {
    pub fn write_input(&mut self, bytes: &[u8]) {
        let _ = self.pty_writer.write_all(bytes);
        let _ = self.pty_writer.flush();
    }
}

impl FmtDisplay for PackageJob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.command)?;
        f.write_char(' ')?;
        match self.targets.len() {
            0 => unreachable!(),
            1 => f.write_str(self.targets[0].as_str())?,
            2 => write!(
                f,
                "{} & {}",
                self.targets[0].as_str(),
                self.targets[0].as_str()
            )?,
            _ => write!(
                f,
                "{} ({} more)",
                self.targets[0].as_str(),
                self.targets.len() - 1
            )?,
        }
        match self.exit_code {
            Some(0) => f.write_str(" ✅")?,
            Some(_) => f.write_str(" ❌")?,
            None => {}
        }
        Ok(())
    }
}
pub struct ExecutionManager {
    next_job_id: JobId,
    pub jobs: HashMap<JobId, PackageJob>,
    pub active_job_order: Vec<JobId>,
}

impl ExecutionManager {
    pub fn new() -> Self {
        Self {
            next_job_id: 1,
            jobs: HashMap::new(),
            active_job_order: Vec::new(),
        }
    }

    pub fn spawn_job(
        &mut self,
        command: &str,
        targets: Vec<PackageName>,
        status_tx: mpsc::Sender<StatusUpdate>,
    ) {
        if targets.is_empty() {
            return;
        }

        // Scan anything not failed to auto close
        if self.active_job_order.len() > 0 {
            for id in self.active_job_order.clone() {
                if self
                    .jobs
                    .get(&id)
                    .is_some_and(|s| s.exit_code.is_some_and(|s| s == 0))
                {
                    self.close_job(&id);
                }
            }
        }

        let job_id = self.next_job_id;
        self.next_job_id += 1;
        let (pty_reader, log_reader, pty_writer, (mut slave_pty, mut pipe_writer)) =
            setup_pty_with_stdin();

        let mut job = PackageJob {
            id: job_id,
            command: command.to_string(),
            targets: targets.clone(),
            logs: Vec::new(),
            buffer: Vec::new(),
            scroll: 0,
            auto_scroll: true,
            exit_code: None,
            pid: None,
            pty_writer,
        };

        spawn_log_reader(pty_reader, job_id, status_tx.clone());
        spawn_log_reader(log_reader, job_id, status_tx.clone());

        let mut cmd = std::process::Command::new(
            std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .join("repo"),
        );
        cmd.arg(command);
        for target in &targets {
            cmd.arg(target.as_str());
        }

        // TODO: How to handle more ANSI code?
        cmd.env("CI", "1");

        self.active_job_order.push(job_id);

        let pipe = (&mut slave_pty, &mut pipe_writer);
        let mut handle = spawn_to_pipe(&mut cmd, &Some(pipe)).unwrap();
        job.pid = Some(handle.id());
        self.jobs.insert(job_id, job);

        let _ = write!(pipe_writer, "{:?}\n", cmd);
        thread::spawn(move || {
            let code = handle.wait().unwrap().code().unwrap_or(-1);
            status_tx
                .send(StatusUpdate::JobFinished(job_id, code))
                .unwrap();
        });
    }

    pub fn handle_status_update(&mut self, update: StatusUpdate) {
        match update {
            StatusUpdate::PushLog(job_id, bytes) => {
                if let Some(job) = self.jobs.get_mut(&job_id) {
                    job.buffer.extend_from_slice(&bytes);
                    let rel = drain_buffer_to_lines(&mut job.buffer, &mut job.logs);
                    if job.auto_scroll {
                        job.scroll += rel;
                    }
                }
            }
            StatusUpdate::JobFinished(job_id, code) => {
                if let Some(job) = self.jobs.get_mut(&job_id) {
                    job.pid.take();
                    job.exit_code = Some(code);
                }
            }
        }
    }

    pub fn close_job(&mut self, id: &JobId) {
        self.jobs.remove(id);
        if let Some(id) = self.active_job_order.iter().position(|s| *s == *id) {
            self.active_job_order.remove(id);
        }
    }
}

fn spawn_log_reader<R>(mut reader: R, job_id: JobId, status_tx: mpsc::Sender<StatusUpdate>)
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut buffer = [0; 1024];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    if status_tx
                        .send(StatusUpdate::PushLog(job_id, buffer[..n].to_vec()))
                        .is_err()
                    {
                        break;
                    }
                }
                Err(e) => {
                    let err_msg = format!("[IO Error] {}", e).into_bytes();
                    let _ = status_tx.send(StatusUpdate::PushLog(job_id, err_msg));
                    break;
                }
            }
        }
    });
}
