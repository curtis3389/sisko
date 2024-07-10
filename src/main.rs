pub mod domain;
pub mod infrastructure;
pub mod ui;

use crate::domain::{
    FileService, IFileService, ISiskoService, ITagService, ITrackService, SiskoService, TagService,
    TrackService,
};
use crate::ui::{CursiveWrapper, ICursive, IUi, UiHost, UiWrapper};
use anyhow::Result;
use clap::Command;
use sisko_lib::id3v2_frame::ID3v2Frame;
use sisko_lib::id3v2_frame_fields::ID3v2FrameFields;
use sisko_lib::id3v2_tag::ID3v2Tag;
use std::fs::File;
use std::io::Write;
use syrette::DIContainer;

/// This is the entrypoint of the program.
fn main() -> Result<()> {
    let container = new_container()?;
    let matches = cli().get_matches();
    match matches.subcommand() {
        None => run_gui(container),
        _ => run_test(),
    }
    Ok(())
}

/// Returns a new dependency injection container.
pub fn new_container() -> Result<DIContainer> {
    let mut container = DIContainer::new();
    container
        .bind::<dyn IFileService>()
        .to::<FileService>()?
        .in_singleton_scope()?;
    container
        .bind::<dyn ITagService>()
        .to::<TagService>()?
        .in_singleton_scope()?;
    container
        .bind::<dyn ITrackService>()
        .to::<TrackService>()?
        .in_singleton_scope()?;
    container
        .bind::<dyn ICursive>()
        .to::<CursiveWrapper>()?
        .in_singleton_scope()?;
    container
        .bind::<dyn IUi>()
        .to::<UiWrapper>()?
        .in_singleton_scope()?;
    container
        .bind::<dyn ISiskoService>()
        .to::<SiskoService>()?
        .in_transient_scope();
    Ok(container)
}

/// Returns a new clap Command for the program's CLI.
pub fn cli() -> Command {
    Command::new("sisko")
        .subcommand_required(false)
        .subcommand(Command::new("test"))
}

/// Runs the program's cursive UI.
pub fn run_gui(container: DIContainer) {
    let ui = UiHost::new(container);
    ui.run();
}

/// Runs a test.
pub fn run_test() {
    let tag = ID3v2Tag::read_from_path("/home/curtis/Downloads/04_discipline_64kb.mp3")
        .expect("Couldn't open test file!");
    println!("{:#?}", tag);
    let mut apic = tag
        .frames
        .iter()
        .filter(|f| f.header.frame_id == "APIC")
        .collect::<Vec<&ID3v2Frame>>();

    if !apic.is_empty() {
        let apic = apic.remove(0);
        match &apic.fields {
            ID3v2FrameFields::AttachedPictureFields {
                picture_data,
                encoding: _,
                mime_type: _,
                picture_type: _,
                description: _,
            } => {
                let mut file = File::create("other.jpg").expect("Couldn't create other.jpg!");
                file.write_all(picture_data)
                    .expect("Error writing data to other.jpg!");
            }
            _ => panic!(),
        }
    }
}
