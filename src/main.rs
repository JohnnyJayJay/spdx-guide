// Copyright (C) 2022  JohnnyJayJay
extern crate core;

mod model;
mod steps;
mod vcs;

use std::io;
use std::path::{PathBuf};
use clap::Parser;
use console::{style, Term};
use dialoguer::theme::ColorfulTheme;
use i18n_embed::DesktopLanguageRequester;
use i18n_embed::fluent::{fluent_language_loader, FluentLanguageLoader};
use i18n_embed_fl::fl;
use rust_embed::RustEmbed;
use crate::model::SpdxDocument;
use crate::steps::SetupData;
use crate::steps::initial_step;
use crate::vcs::git::Git;
use crate::vcs::{Vcs, VcsInfo};


#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {

    /// Directory to run spdx-guide in
    #[clap(short, long, default_value = ".")]
    pub dir: PathBuf,

    /// SPDX file to generate/update, relative to --dir
    #[clap(short, long, default_value = "LICENSE.spdx")]
    file: String,

    /// Update the existing .spdx file (e.g. for a new version)
    #[clap(short, long)]
    update: bool,


}

#[derive(RustEmbed)]
#[folder = "i18n"]
struct Localizations;

fn detect_vcs(path: &PathBuf) -> Option<VcsInfo> {
    let supported_vcs = [Git::open_at];
    for vcs_open in supported_vcs {
        if let Some(vcs) = vcs_open(path) {
            return Some(vcs.read_info())
        }
    }
    None
}

fn main() -> io::Result<()> {
    let mut args = Args::parse();
    args.dir = args.dir.canonicalize().expect("Unable to canonicalize --dir path");

    let mut term = Term::stderr();

    let path = args.dir;
    let language_loader: FluentLanguageLoader = fluent_language_loader!();

    let requested_languages = DesktopLanguageRequester::requested_languages();
    let _result = i18n_embed::select(&language_loader, &Localizations, &requested_languages);
    language_loader.set_use_isolating(false);

    println!("{}", fl!(language_loader, "detecting-vcs", dir = format!("{}", style(path.display()).blue())));
    let vcs_info = detect_vcs(&path);
    let result = match &vcs_info {
        None => fl!(language_loader, "no-vcs"),
        Some(info) => fl!(language_loader, "found-vcs", name = format!("{}", style(&info.vcs_name).green()))
    };
    println!("{}", result);

    //dbg!(&vcs_info);

    let theme = ColorfulTheme::default();
    let mut doc = SpdxDocument::default();
    let mut data = SetupData {
        vcs: vcs_info,
        doc: &mut doc,
        term: &mut term,
        dir: &path,
        filename: args.file,
        i18n: &language_loader,
        theme: &theme
    };
    let mut wrapped_step = Some(initial_step());
    while let Some(ref step) = wrapped_step {
        match step.run(&mut data) {
            Ok(next) => { wrapped_step = next; }
            Err(e) => {
                data.term.clear_line()?;
                data.term.write_line(&format!("{}: {}", fl!(data.i18n, "error"), style(e).red().bold()))?;
            }
        }
    }
    Ok(())

}
