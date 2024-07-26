pub mod domain;
pub mod infrastructure;
pub mod ui;

use crate::domain::SiskoService;
use crate::ui::{CursiveWrapper, UiEvent, UiEventService, UiWrapper};
use anyhow::Result;
use clap::Command;
use domain::LogHistory;
use log::{error, info, LevelFilter};
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

/// This is the entrypoint of the program.
#[tokio::main]
async fn main() -> Result<()> {
    config_logger()?;
    let matches = cli().get_matches();
    match matches.subcommand() {
        None => run_gui().await,
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

pub fn config_logger() -> Result<()> {
    let file_appender = FileAppender::builder()
        .encoder(Box::new(JsonEncoder::new()))
        .build("log.txt")?;
    let log_memory = LogHistory::instance().logs();
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

/// Returns a new clap Command for the program's CLI.
pub fn cli() -> Command {
    Command::new("sisko")
        .subcommand_required(false)
        .subcommand(Command::new("test"))
}

/// Runs the program's cursive UI.
pub async fn run_gui() {
    // create ui
    let cursive = CursiveWrapper::new();

    // bind ui to event-handling thread with a callback
    UiEventService::instance().subscribe(Box::new(move |event| match event {
        UiEvent::OpenLogs => SiskoService::instance().open_logs(),
        UiEvent::FileSelected(file) => SiskoService::instance().add_file(file.clone()).unwrap(),
        UiEvent::FolderSelected(folder) => {
            SiskoService::instance().add_folder(folder.clone()).unwrap()
        }
        UiEvent::OpenAddFile => UiWrapper::instance().open_file_dialog(),
        UiEvent::OpenAddFolder => UiWrapper::instance().open_directory_dialog(),
        UiEvent::ScanAudioFile(audio_file) => {
            let audio_file = audio_file.clone();
            tokio::spawn(async move {
                match SiskoService::instance().scan_audio_file(&audio_file).await {
                    Ok(_) => {}
                    Err(e) => {
                        error!("{}", e);
                        error!("{}", e.root_cause());
                        error!("{}", e.backtrace());
                    }
                }
            });
        }
        UiEvent::SelectAlbumView(album_view) => {
            let album = album_view.album.lock().unwrap();
            if let Some(track_id) = &album_view.track_id {
                let track = album.track(track_id);
                if let Some(audio_file) = track.matched_files.first() {
                    SiskoService::instance().select_audio_file(audio_file)
                }
            }
        }
        UiEvent::SelectClusterFile(audio_file_view) => {
            SiskoService::instance().select_audio_file(&audio_file_view.audio_file)
        }
        UiEvent::SubmitClusterFile(audio_file_view) => {
            UiWrapper::instance().open_audio_file_dialog(audio_file_view)
        }
        UiEvent::SubmitMetadataRow(tag_field_view) => {
            UiWrapper::instance().open_tag_field_dialog(tag_field_view)
        }
    }));

    info!("Running GUI.");
    cursive.run();
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
