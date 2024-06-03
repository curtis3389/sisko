pub mod domain;
pub mod infrastructure;
pub mod ui;

use crate::domain::{
    FileService, IFileService, ISiskoService, ITagService, ITrackService, SiskoService, TagService,
    TrackService,
};
use crate::ui::{CursiveWrapper, ICursive, Ui, UiHost, UiWrapper};
use clap::Command;
use sisko_lib::id3v2_frame::ID3v2Frame;
use sisko_lib::id3v2_frame_fields::ID3v2FrameFields;
use sisko_lib::id3v2_tag::ID3v2Tag;
use std::fs::File;
use std::io::Write;
use syrette::DIContainer;

/// This is the entrypoint of the program.
fn main() {
    let container = new_container();
    let matches = cli().get_matches();
    match matches.subcommand() {
        None => run_gui(container),
        _ => run_test(),
    }
}

/// Returns a new dependency injection container.
pub fn new_container() -> DIContainer {
    let mut container = DIContainer::new();
    container
        .bind::<dyn IFileService>()
        .to::<FileService>()
        .unwrap()
        .in_singleton_scope()
        .unwrap();
    container
        .bind::<dyn ITagService>()
        .to::<TagService>()
        .unwrap()
        .in_singleton_scope()
        .unwrap();
    container
        .bind::<dyn ITrackService>()
        .to::<TrackService>()
        .unwrap()
        .in_singleton_scope()
        .unwrap();
    container
        .bind::<dyn ICursive>()
        .to::<CursiveWrapper>()
        .unwrap()
        .in_singleton_scope()
        .unwrap();
    container
        .bind::<dyn Ui>()
        .to::<UiWrapper>()
        .unwrap()
        .in_singleton_scope()
        .unwrap();
    container
        .bind::<dyn ISiskoService>()
        .to::<SiskoService>()
        .unwrap()
        .in_transient_scope();
    container
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
    let tag = ID3v2Tag::read_from_path("/home/curtis/Downloads/04_discipline_64kb.mp3").unwrap();
    println!("{:#?}", tag);
    let mut apic = tag
        .frames
        .iter()
        .filter(|f| f.header.frame_id == "APIC")
        .collect::<Vec<&ID3v2Frame>>();

    if apic.len() != 0 {
        let apic = apic.remove(0);
        match &apic.fields {
            ID3v2FrameFields::AttachedPictureFields {
                picture_data,
                encoding: _,
                mime_type: _,
                picture_type: _,
                description: _,
            } => {
                let mut file = File::create("other.jpg").unwrap();
                file.write_all(&picture_data).unwrap();
            }
            _ => panic!(),
        }
    }
}
