// Copyright (C) 2022  JohnnyJayJay
use std::cmp::{max, min};
use std::collections::{HashMap};
use std::path::PathBuf;
use git2::{Oid, Repository};
use crate::vcs::{User, VcsInfo, Vcs};

pub struct Git {
    repo: Repository,
}

impl Git {
    fn tagged(&self, commit: Oid) -> Option<String> {
        self.repo.references_glob("refs/tags/*").ok()?
            .filter_map(|r| r.ok())
            .find(|r| r.peel_to_commit().ok().map_or(false, |c| c.id() == commit))
            .and_then(|r| r.name().map(String::from))
    }
}

impl Vcs for Git {
    fn open_at(path: &PathBuf) -> Option<Self> {
        Repository::open(path).ok().map(|r| Git { repo: r })
    }

    fn read_info(&self) -> VcsInfo {
        let config = self.repo.config().ok();
        let username = config.as_ref()
            .and_then(|c| c.get_entry("user.name").ok())
            .and_then(|entry| entry.value().map(String::from));
        let email = config.as_ref()
            .and_then(|c| c.get_entry("user.email").ok())
            .and_then(|entry| entry.value().map(String::from));

        let remote_urls = self.repo.remotes().ok()
            .map_or(vec![], |s| s.into_iter()
                .filter_map(|name| name)
                .filter_map(|name| self.repo.find_remote(name).ok())
                .filter_map(|remote| remote.url().map(String::from))
                .collect());

        let head_tag = self.repo.revwalk()
            .and_then(|mut w| w.push_head().map(|_| w))
            .ok()
            .and_then(|w| w.filter_map(|c| c.ok())
                .find_map(|c| self.tagged(c)));

        let head_ref = self.repo.head().ok()
            .and_then(|reference| reference.name().map(String::from));
        let head_commit = self.repo.head().ok()
            .and_then(|reference| reference.peel_to_commit().ok())
            .map(|commit| commit.id().to_string());

        let mut authors = Vec::new();
        let mut authors_seen = HashMap::new();
        if let Some(log) = head_ref.as_ref().and_then(|h| self.repo.reflog(h).ok()) {
            for log_entry in log.iter() {
                let committer = log_entry.committer();
                if let Some(name) = committer.name() {
                    let name = name.to_string();
                    match authors_seen.remove(&name) {
                        None => {
                            authors_seen.insert(name.clone(), 0u32);
                            authors.push(User { name, email: committer.email().map(String::from) });
                        }
                        Some(count) => {
                            authors_seen.insert(name, count + 1);
                        }
                    }

                }
            }
        }

        let oldest_authors = Vec::from(&authors[max(0, authors.len() as i32 - 5) as usize..]);
        authors.sort_by_key(|u| authors_seen.get(&u.name));
        let active_authors = Vec::from(&authors[..min(5, authors.len())]);

        let version_str = head_tag.as_ref().and_then(|name| name.strip_prefix("refs/tags/").map(String::from));
        VcsInfo {
            vcs_name: "git".to_string(),
            user: username.map(|name| User { name, email }),
            active_project_authors: active_authors,
            oldest_project_authors: oldest_authors,
            remote_urls,
            head_refs: vec![head_ref, head_tag, head_commit].into_iter().filter_map(|h| h).collect(),
            latest_version: version_str
        }
    }
}