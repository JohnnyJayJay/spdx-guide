// Copyright (C) 2022  JohnnyJayJay

use std::fs::File;
use std::io;
use std::io::{Write};
use std::path::{PathBuf};
use console::{style, Term};
use dialoguer::{Confirm, Input, Select};
use dialoguer::theme::Theme;
use i18n_embed::fluent::FluentLanguageLoader;
use i18n_embed_fl::fl;
use uuid::Uuid;
use whoami::{realname, username};
use crate::model::{SpdxDocument};
use crate::vcs::{User, VcsInfo};

pub struct SetupData<'a> {
    pub vcs: Option<VcsInfo>,
    pub creators: Vec<String>,
    pub doc: &'a mut SpdxDocument,
    pub term: &'a mut Term,
    pub filename: String,
    pub dir: &'a PathBuf,
    pub i18n: &'a FluentLanguageLoader,
    pub theme: &'a dyn Theme,
}

pub trait SetupStep: 'static {
    fn run(&self, data: &mut SetupData) -> io::Result<Option<Box<dyn SetupStep>>>;
}

const SPDX_VERSION: &str = "SPDX-2.3";
const TOOL_NAME: &str = env!("CARGO_PKG_NAME");
const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");

fn step<S: SetupStep>(step_object: S) -> io::Result<Option<Box<dyn SetupStep>>> {
    Ok(Some(Box::new(step_object)))
}

pub fn initial_step() -> Box<dyn SetupStep> {
    Box::new(FixedDocumentPropertiesStep)
}

struct FixedDocumentPropertiesStep;

impl SetupStep for FixedDocumentPropertiesStep {
    fn run(&self, data: &mut SetupData) -> io::Result<Option<Box<dyn SetupStep>>> {
        let section = &mut data.doc.document_section;
        section.add_entry("SPDXVersion", SPDX_VERSION);
        section.add_entry("DataLicense", "CC0-1.0");
        section.add_entry("SPDXID", "SPDXRef-DOCUMENT");
        section.add_entry("LicenseListVersion", spdx::identifiers::VERSION);
        section.add_comment("Update DocumentComment if you make further changes to this document");
        section.add_entry("DocumentComment", "This document only gives licensing information about the package it was created for, not its dependencies.");
        section.add_entry("Creator", format!("Tool: {}-{}", TOOL_NAME, TOOL_VERSION));
        step(CreatorPersonStep)
    }
}

/// Display a selection prompt of the provided items + "Other" at the end. If "Other" is selected, the user can enter a custom value in a following input prompt.
fn select_or_input<T: ToString>(data: &SetupData, items: &[T], select_prompt: &str, input_prompt: &str) -> io::Result<Option<String>> {
    let last = items.len();
    let select_res = if items.is_empty() {
        Some(last)
    } else {
        Select::with_theme(data.theme)
            .with_prompt(select_prompt)
            .items(items)
            .item(fl!(data.i18n, "other"))
            .default(0)
            .interact_on_opt(data.term)?
    };

    if let Some(selection) = select_res {
        let mut value = None;
        if selection == last {
            let input_res = Input::<String>::with_theme(data.theme)
                .with_prompt(input_prompt)
                .allow_empty(true)
                .interact_on(data.term)?;
            if !input_res.is_empty() {
                value = Some(input_res);
            }
        } else {
            value = Some(items[selection].to_string());
        }
        Ok(value)
    } else {
        Ok(None)
    }
}

struct CreatorPersonStep;

impl SetupStep for CreatorPersonStep {
    fn run(&self, data: &mut SetupData) -> io::Result<Option<Box<dyn SetupStep>>> {
        let mut items = Vec::new();
        if let Some(user) = data.vcs.as_ref()
            .and_then(|vcs| vcs.user.as_ref().map(User::to_string)) {
            items.push(user);
        }
        items.push(format!("{} ()", username()));
        items.push(format!("{} ()", realname()));

        let select_prompt = &fl!(data.i18n, "creator-person-prompt");
        let input_prompt = &fl!(data.i18n, "creator-custom-person-prompt");
        match select_or_input(data, items.as_slice(), select_prompt, input_prompt)? {
            Some(person) => {
                data.doc.document_section.add_entry("Creator", format!("Person: {}", person));
                data.creators.push(person);
                step(CreatorHasOrgStep)
            }
            None => step(CreatorHasOrgStep)
        }
    }
}

struct CreatorHasOrgStep;

impl SetupStep for CreatorHasOrgStep {
    fn run(&self, data: &mut SetupData) -> io::Result<Option<Box<dyn SetupStep>>> {
        let has_org = Confirm::with_theme(data.theme)
            .with_prompt(fl!(data.i18n, "creator-has-org-prompt"))
            .default(false)
            .interact_on(data.term)?;

        if has_org {
            step(CreatorOrgStep)
        } else {
            step(PackageNameStep)
        }
    }
}

struct CreatorOrgStep;

impl SetupStep for CreatorOrgStep {
    fn run(&self, data: &mut SetupData) -> io::Result<Option<Box<dyn SetupStep>>> {
        let org = Input::<String>::with_theme(data.theme)
            .with_prompt(fl!(data.i18n, "creator-org-prompt"))
            .allow_empty(true)
            .interact_on(data.term)?;
        if !org.is_empty() {
            data.doc.document_section.add_entry("Creator", format!("Organization: {}", org));
            data.creators.push(org);
        }
        step(PackageNameStep)
    }
}

struct PackageNameStep;

impl SetupStep for PackageNameStep {
    fn run(&self, data: &mut SetupData) -> io::Result<Option<Box<dyn SetupStep>>> {
        let name = Input::<String>::with_theme(data.theme)
            .with_prompt(fl!(data.i18n, "name-prompt"))
            .default(data.dir.file_name().and_then(|str| str.to_str()).unwrap_or_default().to_string())
            .interact_on(data.term)?;
        data.doc.package_section.add_entry("SPDXID", format!("SPDXRef-Package-{}", &name));
        data.doc.package_section.add_entry("PackageName", name);
        step(PackageVersionStep)
    }
}

struct PackageVersionStep;

impl SetupStep for PackageVersionStep {
    fn run(&self, data: &mut SetupData) -> io::Result<Option<Box<dyn SetupStep>>> {
        let version = Input::<String>::with_theme(data.theme)
            .with_prompt(fl!(data.i18n, "version-prompt"))
            .with_initial_text(data.vcs.as_ref().and_then(|info| info.latest_version.as_ref().map(|s| s.as_str())).unwrap_or_default())
            .allow_empty(true)
            .interact_on(data.term)?;
        if !version.is_empty() {
            data.doc.package_section.add_entry("PackageVersion", version);
        }
        step(DocumentNameStep)
    }
}

struct DocumentNameStep;

impl SetupStep for DocumentNameStep {
    fn run(&self, data: &mut SetupData) -> io::Result<Option<Box<dyn SetupStep>>> {
        let input = Input::<String>::with_theme(data.theme)
            .with_prompt(fl!(data.i18n, "doc-name-prompt"))
            .default(
                format!("{}{}",
                        data.doc.package_section.find("PackageName")[0],
                        data.doc.package_section.find("PackageVersion")
                            .get(0).map(|v| format!("-{}", v)).unwrap_or_default())
            ).interact_on(data.term)?;
        data.doc.document_section.add_entry("DocumentName", input);
        step(DocumentNamespaceStep)
    }
}

struct DocumentNamespaceStep;

impl SetupStep for DocumentNamespaceStep {
    fn run(&self, data: &mut SetupData) -> io::Result<Option<Box<dyn SetupStep>>> {
        let doc_name = data.doc.document_section.find("DocumentName")[0];
        let random_uuid = Uuid::new_v4();
        let default_namespace = format!("https://spdx.org/spdxdocs/{}-{}", doc_name, random_uuid);
        data.doc.document_section.add_entry("DocumentNamespace", &default_namespace);
        step(PackageSupplierStep)
    }
}


trait AuthorStep: Default {
    fn get_relevant_authors<'a>(&self, vcs: &'a VcsInfo) -> &'a [User];

    fn name(&self) -> String;

    fn next_step(&self) -> Box<dyn SetupStep>;
}

impl<T: AuthorStep + FinishStep + Default + Clone + 'static> SetupStep for T {
    fn run(&self, data: &mut SetupData) -> io::Result<Option<Box<dyn SetupStep>>> {
        let mut items: Vec<String> = data.vcs.as_ref().map(|vcs| self.get_relevant_authors(vcs).into_iter().map(User::to_string).collect()).unwrap_or_default();
        items.extend_from_slice(&data.creators);
        let noassertion = fl!(data.i18n, "no-assertion");
        items.push(noassertion.clone());
        let select_prompt = data.i18n.get(&format!("select-{}-prompt", self.name()));
        let input_prompt = data.i18n.get(&format!("input-{}-prompt", self.name()));
        match select_or_input(data, items.as_slice(), &select_prompt, &input_prompt)? {
            Some(name) => if name == noassertion {
                Ok(Some(self.finish(data, "NOASSERTION".to_string())))
            } else {
                step(PersonOrOrgStep { name, delegate: Box::new(Self::default()) })
            },
            None => Ok(Some(self.next_step()))
        }
    }
}

#[derive(Clone, Default)]
struct PackageSupplierStep;

impl AuthorStep for PackageSupplierStep {
    fn get_relevant_authors<'a>(&self, vcs: &'a VcsInfo) -> &'a [User] {
        vcs.active_project_authors.as_slice()
    }

    fn name(&self) -> String {
        String::from("supplier")
    }

    fn next_step(&self) -> Box<dyn SetupStep> {
        Box::new(PackageOriginatorStep)
    }
}

impl FinishStep for PackageSupplierStep {
    fn finish(&self, data: &mut SetupData, value: String) -> Box<dyn SetupStep> {
        data.doc.package_section.add_entry("PackageSupplier", value);
        Box::new(AskDifferentOriginatorStep)
    }
}

trait FinishStep {
    fn finish(&self, data: &mut SetupData, value: String) -> Box<dyn SetupStep>;
}

enum PersonOrOrgSelect {
    Person,
    Org,
    Back,
}

struct PersonOrOrgStep<S> {
    name: String,
    delegate: Box<S>,
}

impl<'a, S> SetupStep for PersonOrOrgStep<S> where S: SetupStep + FinishStep + Clone {
    fn run(&self, data: &mut SetupData) -> io::Result<Option<Box<dyn SetupStep>>> {
        let items = vec![PersonOrOrgSelect::Person, PersonOrOrgSelect::Org, PersonOrOrgSelect::Back];
        let selection = Select::with_theme(data.theme)
            .with_prompt(fl!(data.i18n, "ask-person-or-org"))
            .item(fl!(data.i18n, "person"))
            .item(fl!(data.i18n, "org"))
            .item(fl!(data.i18n, "go-back"))
            .default(0)
            .interact_on(data.term)?;

        match &items[selection] {
            PersonOrOrgSelect::Back => self.delegate.run(data),
            other => {
                let type_set = match other {
                    PersonOrOrgSelect::Person => "Person",
                    PersonOrOrgSelect::Org => "Organization",
                    PersonOrOrgSelect::Back => panic!("This should never be reached")
                };
                Ok(Some(self.delegate.finish(data, format!("{}: {}", type_set, self.name))))
            }
        }
    }
}


struct AskDifferentOriginatorStep;

impl SetupStep for AskDifferentOriginatorStep {
    fn run(&self, data: &mut SetupData) -> io::Result<Option<Box<dyn SetupStep>>> {
        let choice = Confirm::with_theme(data.theme)
            .with_prompt(fl!(data.i18n, "ask-different-originator-prompt"))
            .default(false)
            .interact_on(data.term)?;

        if choice {
            step(PackageOriginatorStep)
        } else {
            step(DownloadLocationInitialStep)
        }
    }
}

#[derive(Default, Clone)]
struct PackageOriginatorStep;

impl AuthorStep for PackageOriginatorStep {
    fn get_relevant_authors<'a>(&self, vcs: &'a VcsInfo) -> &'a [User] {
        vcs.oldest_project_authors.as_slice()
    }

    fn name(&self) -> String {
        String::from("originator")
    }

    fn next_step(&self) -> Box<dyn SetupStep> {
        Box::new(DownloadLocationInitialStep)
    }
}

impl FinishStep for PackageOriginatorStep {
    fn finish(&self, data: &mut SetupData, value: String) -> Box<dyn SetupStep> {
        data.doc.package_section.add_entry("PackageOriginator", value);
        Box::new(DownloadLocationInitialStep)
    }
}

struct DownloadLocationInitialStep;

impl SetupStep for DownloadLocationInitialStep {
    fn run(&self, data: &mut SetupData) -> io::Result<Option<Box<dyn SetupStep>>> {
        let remotes = data.vcs.as_ref().map(|vcs| vcs.remote_urls.as_slice()).unwrap_or_default();
        let selection = Select::with_theme(data.theme)
            .with_prompt(fl!(data.i18n, "download-select-prompt"))
            .items(remotes)
            .item(fl!(data.i18n, "nowhere"))
            .item(fl!(data.i18n, "no-assertion"))
            .item(fl!(data.i18n, "other"))
            .default(0)
            .interact_on(data.term)?;

        const DIRECT_CHOICES: [&str; 2] = ["NONE", "NOASSERTION"];
        let offset = remotes.len();
        if selection < offset {
            step(AddRevisionToVcsDownloadLocationStep { base_url: remotes[selection].clone() })
        } else if selection < offset + DIRECT_CHOICES.len() {
            data.doc.package_section.add_entry("DownloadLocation", DIRECT_CHOICES[selection]);
            step(DeclaredLicenseStep)
        } else {
            step(OtherDownloadLocationStep)
        }
    }
}

#[derive(Clone)]
struct AddRevisionToVcsDownloadLocationStep {
    base_url: String,
}

impl SetupStep for AddRevisionToVcsDownloadLocationStep {
    fn run(&self, data: &mut SetupData) -> io::Result<Option<Box<dyn SetupStep>>> {
        if let Some(ref vcs) = data.vcs {
            let items = vcs.head_refs.as_slice();
            let select_prompt = fl!(data.i18n, "download-rev-select-prompt");
            let input_prompt = fl!(data.i18n, "download-rev-input-prompt");
            let rev = select_or_input(data, items, &select_prompt, &input_prompt)?;
            data.doc.package_section.add_entry("DownloadLocation", format!("{}+{}{}", vcs.vcs_name, self.base_url, rev.map(|r| format!("@{}", r)).unwrap_or_default()));
            step(DeclaredLicenseStep)
        } else {
            step(DeclaredLicenseStep)
        }
    }
}

struct OtherDownloadLocationStep;

impl SetupStep for OtherDownloadLocationStep {
    fn run(&self, data: &mut SetupData) -> io::Result<Option<Box<dyn SetupStep>>> {
        let url = Input::<String>::with_theme(data.theme)
            .with_prompt(fl!(data.i18n, "other-download-prompt"))
            .interact_on(data.term)?;
        data.doc.package_section.add_entry("DownloadLocation", url);
        step(DeclaredLicenseStep)
    }
}

struct DeclaredLicenseStep;

impl SetupStep for DeclaredLicenseStep {
    fn run(&self, data: &mut SetupData) -> io::Result<Option<Box<dyn SetupStep>>> {
        let license = Input::<String>::with_theme(data.theme)
            .with_prompt(fl!(data.i18n, "license-input-prompt"))
            .allow_empty(true)
            .validate_with(|input: &String|
                (if input.is_empty() { Ok(()) } else { Err("") })
                    .or(spdx::Expression::parse(input)
                        .map(|_| ())
                        .map_err(|err| err.to_string())))
            .interact_on(data.term)?;
        if license.is_empty() {
            data.doc.package_section.add_comment("Edit the line below to specify a license.");
            data.doc.package_section.add_comment("DeclaredLicense: LICENSE-ID");
        } else {
            data.doc.package_section.add_entry("DeclaredLicense", license);
        }
        step(AskVerificationCodeStep)
    }
}

struct AskVerificationCodeStep;

impl SetupStep for AskVerificationCodeStep {
    fn run(&self, data: &mut SetupData) -> io::Result<Option<Box<dyn SetupStep>>> {
        let choice = Confirm::with_theme(data.theme)
            .with_prompt(fl!(data.i18n, "ask-verif-code"))
            .interact_on(data.term)?;
        if choice {
            step(VerificationCodeStep)
        } else {
            step(FileCreateStep)
        }
    }
}

struct VerificationCodeStep;

impl SetupStep for VerificationCodeStep {
    fn run(&self, data: &mut SetupData) -> io::Result<Option<Box<dyn SetupStep>>> {
        data.term.write_line(&format!("{}", style("Sorry, this feature is not yet implemented.").red()))?;
        step(FileCreateStep)
    }
}

struct FileCreateStep;

impl SetupStep for FileCreateStep {
    fn run(&self, data: &mut SetupData) -> io::Result<Option<Box<dyn SetupStep>>> {
        data.term.write_line(&fl!(data.i18n, "creating-file"))?;
        let mut file_path = data.dir.clone();
        file_path.push(&data.filename);
        let mut file = File::create(file_path.as_path())?;
        file.write_all(data.doc.to_string().as_bytes())?;
        Ok(None)
    }
}


