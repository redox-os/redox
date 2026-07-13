use blake3::Hash;
use serde::{Serialize, de::DeserializeOwned};
use std::{
    collections::BTreeSet,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{self, Command, Stdio},
    time::SystemTime,
};
use walkdir::{DirEntry, WalkDir};

use crate::{
    Error, Result, bail_other_err,
    config::translate_mirror,
    cook::pty::{PtyOut, spawn_to_pipe},
    wrap_io_err, wrap_other_err,
};

//TODO: pub(crate) for all of these functions

pub fn remove_all(path: &Path) -> Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
    .map_err(wrap_io_err!(path, "Removing all"))
}

pub fn create_dir(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir).map_err(wrap_io_err!(dir, "Recursively creating dir"))
}

pub fn create_dir_clean(dir: &Path) -> Result<()> {
    if dir.is_dir() {
        remove_all(dir)?;
    }
    create_dir(dir)
}

pub fn create_target_dir(recipe_dir: &Path, target: &'static str) -> Result<PathBuf> {
    let target_dir = recipe_dir.join("target").join(target);
    if !target_dir.is_dir() {
        create_dir(&target_dir)?;
    }
    Ok(target_dir)
}

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn move_dir_all_fn<'a>(
    src: impl AsRef<Path>,
    mv: &'a Box<impl Fn(PathBuf) -> Option<&'a Path>>,
) -> io::Result<()> {
    move_dir_all_inner_fn(&src, &src, mv)
}

fn move_dir_all_inner_fn<'a>(
    src: impl AsRef<Path>,
    srcrel: impl AsRef<Path>,
    mv: &'a Box<impl Fn(PathBuf) -> Option<&'a Path>>,
) -> io::Result<()> {
    let mut files = Vec::new();
    for entry in fs::read_dir(&src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            move_dir_all_inner_fn(entry.path(), srcrel.as_ref(), mv)?;
        } else {
            let path: PathBuf = entry.path();
            let Ok(relpath) = path.strip_prefix(&srcrel) else {
                continue;
            };

            if let Some(dst) = mv(relpath.to_path_buf()) {
                files.push((entry.path(), relpath.to_path_buf(), dst.to_owned()));
            }
        }
    }
    for (src, srcrel, dst) in files {
        let path = dst.join(&srcrel);
        fs::create_dir_all(&path.parent().unwrap())?;
        std::fs::rename(&src, &path)?;
    }
    Ok(())
}

pub fn symlink(original: impl AsRef<Path>, link: impl AsRef<Path>) -> Result<()> {
    std::os::unix::fs::symlink(&original, &link)
        .map_err(wrap_io_err!(link.as_ref(), "Creating symlink"))
}

pub fn modified_is_newer(src: &Path, dst: &Path) -> bool {
    match (fs::metadata(src), fs::metadata(dst)) {
        (Ok(src_meta), Ok(dst_meta)) => match (src_meta.modified(), dst_meta.modified()) {
            (Ok(src_time), Ok(dst_time)) => src_time > dst_time,
            (Ok(_), Err(_)) => true,
            _ => false,
        },
        (Ok(_), Err(_)) => true,
        _ => false,
    }
}

fn modified_inner(path: &Path, metadata: fs::Metadata) -> Result<SystemTime> {
    metadata
        .modified()
        .map_err(wrap_io_err!(path, "Reading modified time"))
}

pub fn modified(path: &Path) -> Result<SystemTime> {
    let metadata = fs::metadata(path).map_err(wrap_io_err!(path, "Reading metadata"))?;
    modified_inner(path, metadata)
}

pub fn modified_all(
    path: &Vec<PathBuf>,
    func: fn(path: &Path) -> Result<SystemTime>,
) -> Result<SystemTime> {
    let mut newest = SystemTime::UNIX_EPOCH;
    for entry_res in path {
        let modified = func(entry_res)?;
        if modified > newest {
            newest = modified;
        }
    }
    Ok(newest)
}

pub fn modified_all_btree<'a>(
    path: impl Iterator<Item = &'a Path>,
    func: fn(path: &Path) -> Result<SystemTime>,
) -> Result<SystemTime> {
    let mut newest = SystemTime::UNIX_EPOCH;
    for entry_res in path {
        let modified = func(entry_res)?;
        if modified > newest {
            newest = modified;
        }
    }
    Ok(newest)
}

fn modified_dir_inner<F: FnMut(&DirEntry) -> bool>(dir: &Path, filter: F) -> Result<SystemTime> {
    let mut newest = modified(dir)?;
    for entry_res in WalkDir::new(dir).into_iter().filter_entry(filter) {
        let entry = entry_res?;
        let meta = entry.metadata()?;
        if meta.is_dir() {
            continue;
        }
        let modified = modified_inner(entry.path(), meta)?;
        if modified > newest {
            newest = modified;
        }
    }
    Ok(newest)
}

pub fn modified_dir(dir: &Path) -> Result<SystemTime> {
    modified_dir_inner(dir, |_| true)
}

pub fn modified_dir_ignore_git(dir: &Path) -> Result<SystemTime> {
    modified_dir_inner(dir, |entry| {
        entry
            .file_name()
            .to_str()
            .map(|s| s != ".git")
            .unwrap_or(true)
    })
}

pub fn check_files_present(dir: &Path, expected_files: &BTreeSet<&str>) -> Result<bool> {
    let entries = fs::read_dir(dir).map_err(wrap_io_err!(dir, "Reading list files"))?;

    let mut matches = 0;
    for entry_res in entries {
        let entry = entry_res.map_err(wrap_io_err!(dir, "Reading file entry"))?;

        let filename = entry.file_name();
        let Some(filename) = filename.to_str() else {
            continue;
        };

        if expected_files.contains(&filename) {
            matches += 1
        } else if filename.starts_with('.') {
            continue;
        } else {
            return Ok(false);
        }
    }

    Ok(matches == expected_files.len())
}

pub fn rename(src: &Path, dst: &Path) -> Result<()> {
    fs::rename(src, dst).map_err(wrap_io_err!(src, dst, "Renaming"))
}

pub fn run_command(mut command: process::Command, stdout_pipe: &PtyOut) -> Result<()> {
    let status = spawn_to_pipe(&mut command, stdout_pipe)?
        .wait()
        .map_err(wrap_io_err!("waiting to exit"))?;

    if !status.success() {
        return Err(Error::Command(command, status));
    }

    Ok(())
}

pub fn run_command_stdin(
    mut command: process::Command,
    stdin_data: &[u8],
    stdout_pipe: &PtyOut,
) -> Result<()> {
    command.stdin(Stdio::piped());
    let mut child = spawn_to_pipe(&mut command, stdout_pipe)?;

    if let Some(ref mut stdin) = child.stdin {
        stdin
            .write_all(stdin_data)
            .map_err(wrap_io_err!("Writing to stdin"))?;
    } else {
        bail_other_err!("stdin is not captured");
    }

    let status = child.wait().map_err(wrap_io_err!("Spawning"))?;

    if !status.success() {
        return Err(Error::Command(command, status));
    }

    Ok(())
}

pub fn serialize_and_write<T: Serialize>(file_path: &Path, content: &T) -> Result<()> {
    let toml_content = toml::to_string(content).map_err(|err| {
        wrap_other_err!(
            "Failed to serialize content for {:?}: {}",
            file_path.display(),
            err
        )()
    })?;

    fs::write(file_path, toml_content).map_err(wrap_io_err!(file_path, "Writing to file"))?;
    Ok(())
}

pub fn read_toml<T: DeserializeOwned>(file_path: &Path) -> Result<T> {
    // TODO: General error rather than from PackageError?
    toml::from_str(&read_to_string(file_path)?)
        .map_err(|e| Error::Package(pkg::PackageError::Parse(e, Some(file_path.to_path_buf()))))
}

pub fn offline_check_exists(path: &PathBuf) -> Result<()> {
    if !path.exists() {
        bail_other_err!(
            "{path:?} is not exist and unable to continue in offline mode",
            path = path.display(),
        );
    }
    Ok(())
}

pub fn download_wget(url: &str, dest: &PathBuf, logger: &PtyOut) -> Result<()> {
    if !dest.is_file() {
        let dest_tmp = PathBuf::from(format!("{}.tmp", dest.display()));
        let mut command = Command::new("wget");
        command.arg(translate_mirror(url));
        command.arg("--continue").arg("-O").arg(&dest_tmp);
        run_command(command, logger)?;
        rename(&dest_tmp, &dest)?;
    }
    Ok(())
}

pub fn read_to_string(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(wrap_io_err!(path, "Reading file"))
}

pub fn get_file_blake3(path: &PathBuf) -> Result<String> {
    get_blake3(path).map(|s| s.to_hex().to_string())
}

fn get_blake3(path: &PathBuf) -> Result<Hash> {
    let mut f = fs::File::open(&path).map_err(wrap_io_err!(path, "Opening file for blake3"))?;
    let hash = blake3::Hasher::new()
        .update_reader(&mut f)
        .map_err(wrap_io_err!(path, "Reading file for blake3"))?
        .finalize();
    Ok(hash)
}

fn get_blake3_str(s: &str) -> Result<Hash> {
    let mut f = s.as_bytes();
    let hash = blake3::Hasher::new()
        .update_reader(&mut f)
        .map_err(wrap_io_err!("Reading string for blake3"))?
        .finalize();
    Ok(hash)
}

/// get combined hashes from files and scripts
pub fn get_combined_blake3(
    dir: &PathBuf,
    files: &Vec<String>,
    contents: &Vec<String>,
) -> Result<String> {
    if files.is_empty() && contents.is_empty() {
        // pkgutils allow omiting the field with empty string
        return Ok("".into());
    }

    let mut combined_bytes: [u8; _] = [0; blake3::OUT_LEN];

    for file in files {
        let hash = get_blake3(&dir.join(file))?;
        let hash_bytes = hash.as_bytes();

        for i in 0..hash_bytes.len() {
            combined_bytes[i] = (combined_bytes[i].rotate_left(2)) ^ hash_bytes[i];
        }
    }
    for s in contents {
        let hash = get_blake3_str(s)?;
        let hash_bytes = hash.as_bytes();

        for i in 0..hash_bytes.len() {
            combined_bytes[i] = (combined_bytes[i].rotate_left(2)) ^ hash_bytes[i];
        }
    }

    Ok(Hash::from(combined_bytes).to_hex().to_string())
}

/// get commit rev and return if it's detached or not
pub fn get_git_head_rev(dir: &PathBuf) -> Result<(String, bool)> {
    let git_head = dir.join(".git/HEAD");
    let head_str = read_to_string(&git_head)?;
    if head_str.starts_with("ref: ") {
        let entry = head_str["ref: ".len()..].trim_end();
        let git_ref = dir.join(".git").join(entry);
        let ref_str = if git_ref.is_file() {
            read_to_string(&git_ref)?
        } else {
            get_git_ref_entry(dir, entry)?
        };
        Ok((ref_str.trim().to_string(), false))
    } else {
        Ok((head_str.trim().to_string(), true))
    }
}

/// get commit from "rev" which either a full commit hash or a tag name
pub fn get_git_tag_rev(dir: &PathBuf, tag: &str, logger: &PtyOut) -> Result<String> {
    if tag.len() == 40 && tag.chars().all(|f| f.is_ascii_hexdigit()) {
        return Ok(tag.to_string());
    }
    let r = get_git_ref_entry(dir, &format!("refs/tags/{tag}"));
    match r {
        Ok(r) => Ok(r),
        Err(_) => {
            // probably need to run "git gc"
            let mut command = Command::new("git");
            command.arg("-C").arg(dir);
            command.arg("gc");
            run_command(command, logger)?;
            get_git_ref_entry(dir, &format!("refs/tags/{tag}"))
        }
    }
}

pub fn get_git_ref_entry(dir: &PathBuf, entry: &str) -> Result<String> {
    // https://git-scm.com/book/en/v2/Git-Internals-Maintenance-and-Data-Recovery
    let git_refs = dir.join(".git/packed-refs");
    let refs_str = read_to_string(&git_refs)?;
    let mut lines = refs_str.lines();
    while let Some(line) = lines.next() {
        if line.contains(entry) {
            let mut sha = line
                .split_whitespace()
                .next()
                .ok_or_else(wrap_other_err!("Packed-refs line is malformed"))?;
            if let Some(next_line) = lines.next() {
                if next_line.starts_with('^') {
                    sha = &next_line[1..];
                }
            }
            return Ok(sha.to_string());
        }
    }

    Err(wrap_other_err!("Could not find a rev for {}", entry)())
}

/// get commit rev after fetch
pub fn get_git_fetch_rev(dir: &PathBuf, remote_url: &str, remote_branch: &str) -> Result<String> {
    let git_fetch_head = dir.join(".git/FETCH_HEAD");

    let fetch_head_content = read_to_string(&git_fetch_head)?;

    let expected_comment_part = format!("branch '{}' of {}", remote_branch, remote_url);

    for line in fetch_head_content.lines() {
        if line.contains(&expected_comment_part) && !line.contains("not-for-merge") {
            let sha = line
                .split_whitespace()
                .next()
                .ok_or_else(wrap_other_err!("FETCH_HEAD line is malformed"))?;

            return Ok(sha.to_string());
        }
    }

    Err(wrap_other_err!(
        "Could not find a fetch target for tracking {}",
        expected_comment_part
    )())
}

#[derive(Default, Debug)]
pub struct GitRemoteTracking {
    /// from .git/HEAD
    pub local_branch: String,
    /// from .git/config -> [branch <local_branch_name>].merge
    pub tracking_branch: String,
    /// from .git/config -> [branch <local_branch_name>].remote (usually "origin")
    pub remote_name: String,
    /// from refs/remotes/<remote_name>/HEAD (the default branch on remote)
    pub remote_branch: String,
    /// from .git/config -> [remote <remote_branch_name>].url
    pub remote_url: String,
}

impl GitRemoteTracking {
    pub fn detached(rev: String) -> Self {
        Self {
            local_branch: rev,
            ..Default::default()
        }
    }
    pub fn check_updated(&self, url: &str, branch: &Option<String>) -> bool {
        if self.local_branch != self.tracking_branch {
            return false;
        }
        if let Some(branch) = branch
            && branch != &self.tracking_branch
        {
            return false;
        }
        if branch.is_none() && self.remote_branch != self.tracking_branch {
            return false;
        }
        if self.remote_name != "origin" || &self.remote_url != chop_dot_git(url) {
            return false;
        }
        true
    }
}

pub fn get_git_remote_tracking(dir: &PathBuf) -> Result<GitRemoteTracking> {
    let git_head = dir.join(".git/HEAD");
    let local_branch = {
        let head_content = read_to_string(&git_head)?;
        if !head_content.starts_with("ref: ") {
            let rev = head_content.trim_end().to_string();
            return Ok(GitRemoteTracking::detached(rev));
        }
        let path = &head_content["ref: ".len()..];
        get_git_branch_name(path.trim_end())?
    };

    let git_config = dir.join(".git/config");
    let config_content = read_to_string(&git_config)?;

    let branch_section = format!("[branch \"{}\"]", local_branch);
    let mut remote_name: Option<String> = None;
    let mut remote_branch: Option<String> = None;
    let mut parsing_branch_section = false;

    for line in config_content.lines().map(|l| l.trim()) {
        if line.is_empty() {
            continue;
        }

        if line == branch_section {
            parsing_branch_section = true;
            continue;
        }

        if parsing_branch_section {
            if line.starts_with('[') {
                break;
            }
            if line.starts_with("remote = ") {
                remote_name = Some(line["remote = ".len()..].trim().to_string());
            }
            if line.starts_with("merge = ") {
                remote_branch = Some(get_git_branch_name(line["merge = ".len()..].trim())?);
            }
        }
    }

    let remote_name = remote_name.ok_or_else(wrap_other_err!(
        "Branch {:?} is not tracking a remote",
        local_branch
    ))?;
    let tracking_branch = remote_branch.unwrap_or("".into());

    let remote_section = format!("[remote \"{}\"]", remote_name);
    let mut remote_url: Option<String> = None;
    let mut parsing_remote_section = false;

    for line in config_content.lines().map(|l| l.trim()) {
        if line.is_empty() {
            continue;
        }

        if line == remote_section {
            parsing_remote_section = true;
            continue;
        }

        if parsing_remote_section {
            if line.starts_with('[') {
                break;
            }
            if line.starts_with("url = ") {
                let mut url = line["url = ".len()..].trim();
                url = chop_dot_git(url);
                remote_url = Some(url.to_string());
            }
        }
    }

    let remote_url = remote_url.ok_or_else(wrap_other_err!(
        "Could not find URL for remote {:?} in .git/config.",
        remote_name
    ))?;

    let remote_branch = get_git_remote_branch(dir, &remote_name)?;
    Ok(GitRemoteTracking {
        local_branch,
        tracking_branch,
        remote_name,
        remote_branch,
        remote_url,
    })
}

pub fn get_git_remote_branch(dir: &PathBuf, remote_name: &str) -> Result<String> {
    let remote_branch = {
        let head_path = format!(".git/refs/remotes/{remote_name}/HEAD");
        let git_remote_head = dir.join(&head_path);
        let head_content = read_to_string(&git_remote_head)?;
        let path = head_content
            .get("ref: ".len()..)
            .ok_or_else(wrap_other_err!(
                "Malformed content {:?} in {head_path}",
                head_content
            ))?;
        get_git_branch_name(path.trim_end())?
    };
    Ok(remote_branch)
}

fn chop_dot_git(url: &str) -> &str {
    if url.ends_with(".git") {
        return &url[..url.len() - ".git".len()];
    }
    url
}

fn get_git_branch_name(branch_path: &str) -> Result<String> {
    // TODO: incorrectly handle branch with slashes
    Ok(branch_path
        .split('/')
        .last()
        .ok_or_else(wrap_other_err!(
            "Failed to parse branch name of {:?}",
            branch_path
        ))?
        .to_string())
}

pub fn get_git_commit_date(dir: &PathBuf) -> Result<String> {
    let mut git = process::Command::new("git");
    git.args(["log", "-1", "--date=iso-strict-local", "--format=%ad"]);
    git.env("TZ", "UTC");
    git.current_dir(dir);
    git.stdout(Stdio::piped());

    git.output()
        .map_err(wrap_io_err!("Executing git log"))
        .map(|s| String::from_utf8_lossy(&s.stdout).trim().to_string())
}

pub fn get_git_rev_before_date(dir: &PathBuf, date: &str) -> Result<String> {
    let mut git = process::Command::new("git");
    git.args(["rev-list", "-n", "1", &format!("--before={}", date), "HEAD"]);
    git.current_dir(dir);
    git.stdout(Stdio::piped());

    let output = git
        .output()
        .map_err(wrap_io_err!("Executing git rev-list"))?;
    let rev = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if rev.is_empty() {
        return Err(Error::from(format!(
            "No commit found before {} in {:?}",
            date, dir
        )));
    }
    Ok(rev)
}
