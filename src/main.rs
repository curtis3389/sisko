pub mod domain;
pub mod infrastructure;
pub mod ui;

use crate::domain::events::DomainEvent;
use crate::domain::models::{TagFieldId, TagId};
use crate::domain::repos::{AudioFileRepository, TagRepository, TrackRepository};
use crate::domain::services::{LogHistory, MediatorService, SiskoService};
use crate::ui::events::UiEvent;
use crate::ui::models::AlbumViewId;
use crate::ui::services::{CursiveWrapper, Ui, UiEventService};
use anyhow::{anyhow, Result};
use clap::Command;
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
        None => run_gui().await?,
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
        Ok(())
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
        let mut memory = self
            .memory
            .lock()
            .map_err(|_| anyhow!("Error locking memory mutex!"))?;
        let mut writer = MemoryWriter::new(&mut memory);
        self.encoder.encode(&mut writer, record)?;
        Ok(())
    }

    fn flush(&self) {}
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
pub async fn run_gui() -> Result<()> {
    // create ui
    let cursive = CursiveWrapper::new();

    // bind ui to event-handling thread with a callback
    UiEventService::instance().subscribe(Box::new(move |event| {
        let result = match event {
            UiEvent::OpenLogs => SiskoService::instance().open_logs(),
            UiEvent::FileSelected(file) => {
                let file = file.clone();
                tokio::spawn(async move {
                    if let Err(e) = SiskoService::instance().add_file(file).await {
                        error!("{}", e);
                    }
                });
                Ok(())
            }
            UiEvent::FolderSelected(folder) => {
                let folder = folder.clone();
                tokio::spawn(async move {
                    if let Err(e) = SiskoService::instance().add_folder(folder).await {
                        error!("{}", e);
                    }
                });
                Ok(())
            }
            UiEvent::OpenAddFile => Ui::instance().menu.open_file_dialog(),
            UiEvent::OpenAddFolder => Ui::instance().menu.open_directory_dialog(),
            UiEvent::SaveAudioFile(audio_file) => {
                let audio_file = audio_file.clone();
                tokio::spawn(async move {
                    if let Err(e) = SiskoService::instance().save_audio_file(&audio_file).await {
                        error!("{}", e);
                    }
                });
                Ok(())
            }
            UiEvent::ScanAudioFile(audio_file) => {
                let audio_file = audio_file.clone();
                tokio::spawn(async move {
                    if let Err(e) = SiskoService::instance().scan_audio_file(&audio_file).await {
                        error!("{}", e);
                    }
                });
                Ok(())
            }
            UiEvent::SelectAlbumView(album_view) => {
                let album_view = album_view.clone();
                tokio::spawn(async move {
                    match &album_view.id {
                        AlbumViewId::Album(_album_id) => {
                            // TODO: do something on select an album row?
                        }
                        AlbumViewId::Track(track_id) => {
                            if let Ok(track) =
                                TrackRepository::instance().get_by_key(track_id).await
                            {
                                if let Some(audio_file) = AudioFileRepository::instance()
                                    .get_matched(&track)
                                    .await
                                    .ok()
                                    .and_then(|files| files.into_iter().next())
                                {
                                    if let Err(e) = SiskoService::instance()
                                        .select_audio_file(&audio_file.id)
                                        .await
                                    {
                                        error!("{}", e);
                                    }
                                }
                                // TODO: add select unmatched tracks & album
                            }
                        }
                    }
                });
                Ok(())
            }
            UiEvent::SelectClusterFile(audio_file_view) => {
                let audio_file_view = audio_file_view.clone();
                tokio::spawn(async move {
                    if let Err(e) = SiskoService::instance()
                        .select_audio_file(&audio_file_view.id)
                        .await
                    {
                        error!("{}", e);
                    }
                });
                Ok(())
            }
            UiEvent::SubmitAlbumView(album_view) => {
                let album_view = album_view.clone();
                tokio::spawn(async move {
                    let audio_file = match &album_view.id {
                        AlbumViewId::Album(_) => None,
                        AlbumViewId::Track(track_id) => {
                            match TrackRepository::instance().get_by_key(track_id).await {
                                Ok(track) => {
                                    match AudioFileRepository::instance().get_matched(&track).await
                                    {
                                        Ok(audio_files) => audio_files.into_iter().next(),
                                        Err(e) => {
                                            error!("{}", e);
                                            None
                                        }
                                    }
                                }
                                Err(e) => {
                                    error!("{}", e);
                                    None
                                }
                            }
                        }
                    };
                    if let Err(e) = {
                        Ui::instance()
                            .album_table
                            .open_album_view_dialog(&audio_file, &album_view.title)
                    } {
                        error!("{}", e);
                    }
                });
                Ok(())
            }
            UiEvent::SubmitClusterFile(audio_file_view) => {
                let audio_file_view = audio_file_view.clone();
                tokio::spawn(async move {
                    match AudioFileRepository::instance()
                        .get(&audio_file_view.id)
                        .await
                    {
                        Ok(audio_file) => {
                            let title = audio_file_view.title;
                            if let Err(e) = Ui::instance()
                                .cluster_table
                                .open_audio_file_dialog(&audio_file, &title)
                            {
                                error!("{}", e);
                            }
                        }
                        Err(e) => {
                            error!("{}", e);
                        }
                    }
                });
                Ok(())
            }
            UiEvent::SubmitMetadataRow(tag_field_view) => {
                let tag_field_view = tag_field_view.clone();
                tokio::spawn(async move {
                    let TagFieldId {
                        tag_id:
                            TagId {
                                audio_file_id,
                                tag_type,
                            },
                        tag_field_type,
                    } = &tag_field_view.id;
                    match AudioFileRepository::instance().get(audio_file_id).await {
                        Ok(audio_file) => {
                            match TagRepository::instance().get(&audio_file, tag_type).await {
                                Ok(tag) => {
                                    let field =
                                        tag.get_field(tag_field_type.clone()).unwrap().clone();
                                    if let Err(e) = Ui::instance()
                                        .metadata_table
                                        .open_tag_field_dialog(audio_file, *tag_type, field)
                                    {
                                        error!("{}", e);
                                    }
                                }
                                Err(e) => error!("{}", e),
                            }
                        }
                        Err(e) => error!("{}", e),
                    }
                });
                Ok(())
            }
        };
        if let Err(error) = result {
            error!("Error processing event {event}: {error}!");
        }
    }))?;

    // bind domain events to the ui
    MediatorService::instance().add_handler(Box::new(|event| {
        match event {
            DomainEvent::AudioFileAdded(audio_file) => {
                SiskoService::instance().handle_audio_file_added(audio_file);
            }
            DomainEvent::AudioFileUpdated(audio_file) => {
                SiskoService::instance().handle_audio_file_updated(audio_file);
            }
            DomainEvent::TagUpdated(tag) => {
                SiskoService::instance().handle_tag_updated(tag);
            }
        }
        Ok(())
    }));

    info!("Running GUI.");
    cursive.run()
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
