// Copyright (C) 2022  JohnnyJayJay

use std::path::PathBuf;

pub(crate) mod git;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct User {
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug)]
pub struct VcsInfo {
    pub vcs_name: String,
    pub user: Option<User>,
    pub active_project_authors: Vec<User>,
    pub oldest_project_authors: Vec<User>,
    pub remote_urls: Vec<String>,
    pub head_refs: Vec<String>,
    pub latest_version: Option<String>,
}

pub trait Vcs: Sized {
    fn open_at(path: &PathBuf) -> Option<Self>;

    fn read_info(&self) -> VcsInfo;
}

impl ToString for User {
    fn to_string(&self) -> String {
        format!("{}{}", self.name, self.email.as_ref()
            .map_or_else(|| "".to_string(), |email| format!(" ({})", email)))
    }
}

impl VcsInfo {

    pub fn download_locations(&self, remote_url: &str) -> Vec<String> {
        let mut result = Vec::with_capacity(self.head_refs.len() + 1);
        result.push(format!("{}+{}", self.vcs_name, remote_url));
        result.extend((&self.head_refs).into_iter()
            .map(|r| format!("{}+{}@{}", self.vcs_name, remote_url, r)));
        result
    }
}