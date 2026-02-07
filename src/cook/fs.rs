use serde::Serialize;
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
    config::translate_mirror,
    cook::pty::{PtyOut, spawn_to_pipe},
};

//TODO: pub(crate) for all of these functions

pub fn remove_all(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
    .map_err(|err| format!("failed to remove '{}': {}\n{:?}", path.display(), err, err))
}

pub fn create_dir(dir: &Path) -> Result<(), String> {
    fs::create_dir(dir)
        .map_err(|err| format!("failed to create '{}': {}\n{:?}", dir.display(), err, err))
}

pub fn create_dir_clean(dir: &Path) -> Result<(), String> {
    if dir.is_dir() {
        remove_all(dir)?;
    }
    fs::create_dir_all(dir)
        .map_err(|err| format!("failed to create '{}': {}\n{:?}", dir.display(), err, err))
}

pub fn create_target_dir(recipe_dir: &Path, target: &'static str) -> Result<PathBuf, String> {
    let target_parent_dir = recipe_dir.join("target");
    if !target_parent_dir.is_dir() {
        create_dir(&target_parent_dir)?;
    }
    let target_dir = target_parent_dir.join(target);
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

pub fn symlink(original: impl AsRef<Path>, link: impl AsRef<Path>) -> Result<(), String> {
    std::os::unix::fs::symlink(&original, &link).map_err(|err| {
        format!(
            "failed to symlink '{}' to '{}': {}\n{:?}",
            original.as_ref().display(),
            link.as_ref().display(),
            err,
            err
        )
    })
}

pub fn modified(path: &Path) -> Result<SystemTime, String> {
    let metadata = fs::metadata(path).map_err(|err| {
        format!(
            "failed to get metadata of '{}': {}\n{:#?}",
            path.display(),
            err,
            err
        )
    })?;
    metadata.modified().map_err(|err| {
        format!(
            "failed to get modified time of '{}': {}\n{:#?}",
            path.display(),
            err,
            err
        )
    })
}

pub fn modified_all(
    path: &Vec<PathBuf>,
    func: fn(path: &Path) -> Result<SystemTime, String>,
) -> Result<SystemTime, String> {
    let mut newest = SystemTime::UNIX_EPOCH;
    for entry_res in path {
        let modified = func(entry_res)?;
        if modified > newest {
            newest = modified;
        }
    }
    Ok(newest)
}

pub fn modified_dir_inner<F: FnMut(&DirEntry) -> bool>(
    dir: &Path,
    filter: F,
) -> io::Result<SystemTime> {
    let mut newest = fs::metadata(dir)?.modified()?;
    for entry_res in WalkDir::new(dir).into_iter().filter_entry(filter) {
        let entry = entry_res?;
        let modified = entry.metadata()?.modified()?;
        if modified > newest {
            newest = modified;
        }
    }
    Ok(newest)
}

pub fn modified_dir(dir: &Path) -> Result<SystemTime, String> {
    modified_dir_inner(dir, |_| true).map_err(|err| {
        format!(
            "failed to get modified time of '{}': {}\n{:#?}",
            dir.display(),
            err,
            err
        )
    })
}

pub fn modified_dir_ignore_git(dir: &Path) -> Result<SystemTime, String> {
    modified_dir_inner(dir, |entry| {
        entry
            .file_name()
            .to_str()
            .map(|s| s != ".git")
            .unwrap_or(true)
    })
    .map_err(|err| {
        format!(
            "failed to get modified time of '{}': {}\n{:#?}",
            dir.display(),
            err,
            err
        )
    })
}

pub fn check_files_present(dir: &Path, expected_files: &BTreeSet<&str>) -> Result<bool, String> {
    let entries = fs::read_dir(dir)
        .map_err(|err| format!("failed to get list files of '{}': {:?}", dir.display(), err))?;

    let mut matches = 0;
    for entry_res in entries {
        let entry = entry_res
            .map_err(|err| format!("failed to get file entry of '{}': {:?}", dir.display(), err))?;

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

pub fn rename(src: &Path, dst: &Path) -> Result<(), String> {
    fs::rename(src, dst).map_err(|err| {
        format!(
            "failed to rename '{}' to '{}': {}\n{:?}",
            src.display(),
            dst.display(),
            err,
            err
        )
    })
}

pub fn run_command(mut command: process::Command, stdout_pipe: &PtyOut) -> Result<(), String> {
    let status = spawn_to_pipe(&mut command, stdout_pipe)
        .map_err(|err| format!("failed to run {:?}: {}\n{:#?}", command, err, err))?
        .wait()
        .map_err(|err| format!("failed to run {:?}: {}\n{:#?}", command, err, err))?;

    if !status.success() {
        return Err(format!(
            "failed to run {:?}: exited with status {}",
            command, status
        ));
    }

    Ok(())
}

pub fn run_command_stdin(
    mut command: process::Command,
    stdin_data: &[u8],
    stdout_pipe: &PtyOut,
) -> Result<(), String> {
    command.stdin(Stdio::piped());
    let mut child = spawn_to_pipe(&mut command, stdout_pipe)
        .map_err(|err| format!("failed to spawn {:?}: {}\n{:#?}", command, err, err))?;

    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(stdin_data).map_err(|err| {
            format!(
                "failed to write stdin of {:?}: {}\n{:#?}",
                command, err, err
            )
        })?;
    } else {
        return Err(format!("failed to find stdin of {:?}", command));
    }

    let status = child
        .wait()
        .map_err(|err| format!("failed to run {:?}: {}\n{:#?}", command, err, err))?;

    if !status.success() {
        return Err(format!(
            "failed to run {:?}: exited with status {}",
            command, status
        ));
    }

    Ok(())
}

pub fn serialize_and_write<T: Serialize>(file_path: &Path, content: &T) -> Result<(), String> {
    let toml_content = toml::to_string(content).map_err(|err| {
        format!(
            "Failed to serialize content for '{}': {}",
            file_path.display(),
            err
        )
    })?;

    fs::write(file_path, toml_content)
        .map_err(|err| format!("Failed to write to file '{}': {}", file_path.display(), err))?;
    Ok(())
}

pub fn offline_check_exists(path: &PathBuf) -> Result<(), String> {
    if !path.exists() {
        return Err(format!(
            "'{path}' does not exist and unable to continue in offline mode",
            path = path.display(),
        ))?;
    }
    Ok(())
}

pub fn download_wget(url: &str, dest: &PathBuf, logger: &PtyOut) -> Result<(), String> {
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

/// get commit rev and return if it's detached or not
pub fn get_git_head_rev(dir: &PathBuf) -> Result<(String, bool), String> {
    let git_head = dir.join(".git/HEAD");
    let head_str = fs::read_to_string(&git_head)
        .map_err(|e| format!("unable to read {path}: {e}", path = git_head.display()))?;
    if head_str.starts_with("ref: ") {
        let entry = head_str["ref: ".len()..].trim_end();
        let git_ref = dir.join(".git").join(entry);
        let ref_str = if git_ref.is_file() {
            fs::read_to_string(&git_ref)
                .map_err(|e| format!("unable to read {path}: {e}", path = git_ref.display()))?
        } else {
            get_git_ref_entry(dir, entry)?
        };
        Ok((ref_str.trim().to_string(), false))
    } else {
        Ok((head_str.trim().to_string(), true))
    }
}

/// get commit from "rev" which either a full commit hash or a tag name
pub fn get_git_tag_rev(dir: &PathBuf, tag: &str) -> Result<String, String> {
    if tag.len() == 40 && tag.chars().all(|f| f.is_ascii_hexdigit()) {
        return Ok(tag.to_string());
    }
    get_git_ref_entry(dir, &format!("refs/tags/{tag}"))
}
pub fn get_git_ref_entry(dir: &PathBuf, entry: &str) -> Result<String, String> {
    let git_refs = dir.join(".git/packed-refs");
    let refs_str = fs::read_to_string(&git_refs)
        .map_err(|e| format!("unable to read {path}: {e}", path = git_refs.display()))?;
    for line in refs_str.lines() {
        if line.contains(entry) {
            let sha = line
                .split_whitespace()
                .next()
                .ok_or_else(|| "packed-refs line is malformed.".to_string())?;

            return Ok(sha.to_string());
        }
    }

    Err(format!("Could not find a rev for {}", entry))
}

/// get commit rev after fetch
pub fn get_git_fetch_rev(
    dir: &PathBuf,
    remote_url: &str,
    remote_branch: &str,
) -> Result<String, String> {
    let git_fetch_head = dir.join(".git/FETCH_HEAD");

    let fetch_head_content = fs::read_to_string(&git_fetch_head).map_err(|e| {
        format!(
            "unable to read {path}: {e}",
            path = git_fetch_head.display()
        )
    })?;

    let expected_comment_part = format!("branch '{}' of {}", remote_branch, remote_url);

    for line in fetch_head_content.lines() {
        if line.contains(&expected_comment_part) && !line.contains("not-for-merge") {
            let sha = line
                .split_whitespace()
                .next()
                .ok_or_else(|| "FETCH_HEAD line is malformed.".to_string())?;

            return Ok(sha.to_string());
        }
    }

    Err(format!(
        "Could not find a fetch target for tracking {}",
        expected_comment_part
    ))
}

/// (local_branch_name, remote_branch, remote_name, remote_url)
///    -> ("fix_stuff", "master", "origin", "https://gitlab.redox-os.org/willnode/redox")
pub fn get_git_remote_tracking(dir: &PathBuf) -> Result<(String, String, String, String), String> {
    let git_head = dir.join(".git/HEAD");
    let git_config = dir.join(".git/config");

    let head_content = fs::read_to_string(&git_head)
        .map_err(|e| format!("unable to read {path}: {e}", path = git_head.display()))?;

    if !head_content.starts_with("ref: ") {
        let sha = head_content.trim_end().to_string();
        return Ok((sha, "".to_string(), "".to_string(), "".to_string()));
    }

    let local_branch_path = head_content["ref: ".len()..].trim_end();
    let local_branch_name = get_git_branch_name(local_branch_path)?;

    let config_content = fs::read_to_string(&git_config)
        .map_err(|e| format!("unable to read {path}: {e}", path = git_config.display()))?;

    let branch_section = format!("[branch \"{}\"]", local_branch_name);
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

    let remote_name_str = remote_name
        .ok_or_else(|| format!("Branch '{}' is not tracking a remote.", local_branch_name))?;
    let remote_branch_str = remote_branch.unwrap_or("".into());

    let remote_section = format!("[remote \"{}\"]", remote_name_str);
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

    let remote_url_str = remote_url.ok_or_else(|| {
        format!(
            "Could not find URL for remote '{}' in .git/config.",
            remote_name_str
        )
    })?;

    Ok((
        local_branch_name,
        remote_branch_str,
        remote_name_str,
        remote_url_str,
    ))
}

pub(crate) fn chop_dot_git(url: &str) -> &str {
    if url.ends_with(".git") {
        return &url[..url.len() - ".git".len()];
    }
    url
}

fn get_git_branch_name(local_branch_path: &str) -> Result<String, String> {
    // TODO: incorrectly handle branch with slashes
    Ok(local_branch_path
        .split('/')
        .last()
        .ok_or_else(|| format!("Failed to parse branch name of {:?}", local_branch_path))?
        .to_string())
}
