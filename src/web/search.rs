use serde::Serialize;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::{Result, wrap_io_err};

#[derive(Serialize, Debug, Clone)]
#[serde(untagged)]
enum FsNode {
    Dir(BTreeMap<String, FsNode>),
    File(String),
}

pub struct FileIndexBuilder {
    inner: BTreeMap<String, FsNode>,
}
impl FileIndexBuilder {
    pub const fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }
    fn insert_node(&mut self, path_stack: &[String], name: &str, pkg_name: Option<&str>) {
        let mut current = &mut self.inner;
        for dir in path_stack {
            let node = current
                .entry(dir.clone())
                .or_insert_with(|| FsNode::Dir(BTreeMap::new()));

            if let FsNode::File(_) = node {
                // previously a file, silently replace to dir
                *node = FsNode::Dir(BTreeMap::new());
            }

            match node {
                FsNode::Dir(map) => current = map,
                _ => unreachable!(),
            }
        }

        if let Some(pkg) = pkg_name {
            current
                .entry(name.to_string())
                .and_modify(|node| {
                    if let FsNode::File(existing_pkgs) = node {
                        existing_pkgs.push(',');
                        existing_pkgs.push_str(pkg);
                    } else {
                        *node = FsNode::File(pkg.to_string());
                    }
                })
                .or_insert_with(|| FsNode::File(pkg.to_string()));
        } else {
            current
                .entry(name.to_string())
                .or_insert_with(|| FsNode::Dir(BTreeMap::new()));
        }
    }
    pub fn parse(&mut self, pkg: &str, content: &str) {
        let mut path_stack: Vec<String> = Vec::new();

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let mut prefix_chars: usize = 0;
            let mut name_start_byte = 0;

            for (idx, c) in line.char_indices() {
                if "│├└─ ".contains(c) {
                    prefix_chars += 1;
                } else {
                    name_start_byte = idx;
                    break;
                }
            }

            if name_start_byte == 0 && prefix_chars == 0 {
                continue;
            }

            let depth = (prefix_chars / 4).saturating_sub(1);
            let remainder = line[name_start_byte..].trim();

            path_stack.truncate(depth);

            if remainder.ends_with('/') {
                let name = remainder.trim_end_matches('/');
                self.insert_node(&path_stack, name, None);
                path_stack.push(name.to_string());
            } else {
                let name = if let Some(paren_idx) = remainder.rfind('(') {
                    // Strip size
                    remainder[..paren_idx].trim()
                } else {
                    remainder
                };
                self.insert_node(&path_stack, name, Some(pkg));
            }
        }
    }
    pub fn write(&self, path: &Path) -> Result<()> {
        // JSON: because javascript have them natively
        let json_output = serde_json::to_string(&self.inner).unwrap();
        let mut file = File::create(path).map_err(wrap_io_err!(path, "Opening file"))?;
        file.write_all(json_output.as_bytes())
            .map_err(wrap_io_err!(path, "Writing file"))?;
        Ok(())
    }
}
