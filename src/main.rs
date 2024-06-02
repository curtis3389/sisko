pub mod file;
pub mod file_column;
pub mod file_dialog_type;
pub mod file_service;
pub mod file_type;
pub mod icursive;
pub mod sisko_service;
pub mod tag;
pub mod tag_column;
pub mod tag_service;
pub mod track;
pub mod track_column;
pub mod track_service;
pub mod ui;
pub mod ui_element;

use crate::file_service::{FileService, IFileService};
use crate::icursive::*;
use crate::sisko_service::{ISiskoService, SiskoService};
use crate::tag_service::{ITagService, TagService};
use crate::track_service::{ITrackService, TrackService};
use crate::ui::{IUi, Ui, UiWrapper};
use clap::Command;
use sisko_lib::id3v2_frame::ID3v2Frame;
use sisko_lib::id3v2_frame_fields::ID3v2FrameFields;
use sisko_lib::id3v2_tag::ID3v2Tag;
use std::fs::File as FsFile;
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
        .bind::<dyn IUi>()
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
    let ui = Ui::new(container);
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
                let mut file = FsFile::create("other.jpg").unwrap();
                file.write_all(&picture_data).unwrap();
            }
            _ => panic!(),
        }
    }
}
