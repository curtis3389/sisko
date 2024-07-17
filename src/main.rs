pub mod domain;
pub mod infrastructure;
pub mod ui;

use crate::domain::{
    FileService, IFileService, ISiskoService, ITagService, ITrackService, SiskoService, TagService,
    TrackService,
};
use crate::infrastructure::DIContainerExtensions;
use crate::ui::{CursiveWrapper, ICursive, IUi, UiHost, UiWrapper};
use anyhow::Result;
use clap::Command;
use domain::{ILogHistory, LogHistory};
use infrastructure::acoustid::{AcoustIdService, IAcoustIdService};
use infrastructure::musicbrainz::{IMusicBrainzService, MusicBrainzService};
use log::{info, LevelFilter};
use log4rs::append::file::FileAppender;
use log4rs::append::Append;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::json::JsonEncoder;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::encode::Encode;
use sisko_lib::id3v2_frame::ID3v2Frame;
use sisko_lib::id3v2_frame_fields::ID3v2FrameFields;
use sisko_lib::id3v2_tag::ID3v2Tag;
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};
use syrette::DIContainer;

/// This is the entrypoint of the program.
fn main() -> Result<()> {
    let container = new_container()?;
    {
        let log_history = container.expect_singleton::<dyn ILogHistory>();
        let log_memory = log_history.logs();
        config_logger(log_memory)?;
    }
    let matches = cli().get_matches();
    match matches.subcommand() {
        None => run_gui(container),
        _ => run_test(),
    }
    Ok(())
}

pub struct MemoryWriter<'a> {
    memory: &'a mut Vec<String>,
}

impl<'a> MemoryWriter<'a> {
    pub fn new(memory: &'a mut Vec<String>) -> Self {
        Self { memory }
    }
}

impl<'a> std::io::Write for MemoryWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let s = String::from_utf8(buf.to_vec()).map_err(std::io::Error::other)?;
        self.memory.push(s);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        todo!()
    }
}

impl<'a> log4rs::encode::Write for MemoryWriter<'a> {}

#[derive(Debug)]
pub struct MemoryAppender {
    encoder: Box<dyn Encode>,
    memory: Arc<Mutex<Vec<String>>>,
}

impl MemoryAppender {
    pub fn new(encoder: Box<dyn Encode>, memory: Arc<Mutex<Vec<String>>>) -> Self {
        Self { encoder, memory }
    }
}

impl Append for MemoryAppender {
    fn append(&self, record: &log::Record) -> anyhow::Result<()> {
        let mut memory = self.memory.lock().unwrap();
        let mut writer = MemoryWriter::new(&mut memory);
        self.encoder.encode(&mut writer, record)?;
        Ok(())
    }

    fn flush(&self) {
        todo!()
    }
}

pub fn config_logger(log_memory: Arc<Mutex<Vec<String>>>) -> Result<()> {
    let file_appender = FileAppender::builder()
        .encoder(Box::new(JsonEncoder::new()))
        .build("log.txt")?;
    let memory_appender = MemoryAppender::new(
        Box::new(PatternEncoder::new("[{d}] {l} {m}{n}")),
        log_memory,
    );

    let config = Config::builder()
        .appender(Appender::builder().build("file_appender", Box::new(file_appender)))
        .appender(Appender::builder().build("memory_appender", Box::new(memory_appender)))
        .build(
            Root::builder()
                .appender("file_appender")
                .appender("memory_appender")
                .build(LevelFilter::Info),
        )?;

    log4rs::init_config(config)?;
    Ok(())
}

/// Returns a new dependency injection container.
pub fn new_container() -> Result<DIContainer> {
    let mut container = DIContainer::new();
    container
        .bind::<dyn IAcoustIdService>()
        .to::<AcoustIdService>()?
        .in_singleton_scope()?;
    container
        .bind::<dyn ICursive>()
        .to::<CursiveWrapper>()?
        .in_singleton_scope()?;
    container
        .bind::<dyn IFileService>()
        .to::<FileService>()?
        .in_singleton_scope()?;
    container
        .bind::<dyn ILogHistory>()
        .to::<LogHistory>()?
        .in_singleton_scope()?;
    container
        .bind::<dyn IMusicBrainzService>()
        .to::<MusicBrainzService>()?
        .in_singleton_scope()?;
    container
        .bind::<dyn ISiskoService>()
        .to::<SiskoService>()?
        .in_transient_scope();
    container
        .bind::<dyn ITagService>()
        .to::<TagService>()?
        .in_singleton_scope()?;
    container
        .bind::<dyn ITrackService>()
        .to::<TrackService>()?
        .in_singleton_scope()?;
    container
        .bind::<dyn IUi>()
        .to::<UiWrapper>()?
        .in_singleton_scope()?;
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
    info!("Running GUI.");
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
