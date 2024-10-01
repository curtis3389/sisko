use super::{CbSinkService, UiEventService};
use crate::{
    domain::models::{AudioFile, Metadata},
    infrastructure::TableViewExtensions,
    ui::{
        events::UiEvent,
        models::{AudioFileColumn, AudioFileView, CLUSTER_FILE_TABLE},
    },
};
use anyhow::{anyhow, Result};
use cursive::{
    views::{Button, Dialog, LinearLayout},
    Cursive,
};
use cursive_table_view::TableView;
use log::error;

pub struct ClusterTable {}

impl ClusterTable {
    pub fn new() -> Self {
        Self {}
    }

    pub fn add_cluster_file(&self, audio_file: AudioFile, metadata: &Metadata) -> Result<()> {
        let audio_file_view = AudioFileView::new(&audio_file, metadata);
        CbSinkService::instance()?
            .send(Box::new(move |s: &mut Cursive| {
                s.call_on_name(
                    CLUSTER_FILE_TABLE,
                    |table: &mut TableView<AudioFileView, AudioFileColumn>| match table
                        .index_of(|i| i.id == audio_file_view.id)
                    {
                        Some(index) => {
                            table.remove_item(index);
                            table.insert_item_at(index, audio_file_view.clone());
                        }
                        None => {
                            table.insert_item(audio_file_view.clone());
                            if table.len() == 1 {
                                if let Err(e) = UiEventService::instance()
                                    .send(UiEvent::SelectClusterFile(audio_file_view))
                                {
                                    error!("Error sending select cluster file event: {e}!");
                                };
                            }
                        }
                    },
                );
            }))
            .map_err(|_| anyhow!("Error sending add cluster file callback to CbSink!"))
    }

    pub fn open_audio_file_dialog(&self, audio_file: &AudioFile, title: &str) -> Result<()> {
        let audio_file = audio_file.clone();
        let title = title.to_owned();
        CbSinkService::instance()?
            .send(Box::new(move |s: &mut Cursive| {
                audio_file_dialog(s, audio_file, title);
            }))
            .map_err(|_| anyhow!("Error sending open audio file dialog callback to CbSink!"))
    }

    pub fn remove_cluster_file(&self, audio_file: &AudioFile) -> Result<()> {
        let path = audio_file.id.clone();
        CbSinkService::instance()?
            .send(Box::new(move |s: &mut Cursive| {
                s.call_on_name(
                    CLUSTER_FILE_TABLE,
                    |table: &mut TableView<AudioFileView, AudioFileColumn>| {
                        if let Some((index, _)) = table
                            .borrow_items()
                            .iter()
                            .enumerate()
                            .find(|(_, item)| item.id == path)
                        {
                            table.remove_item(index);
                        }
                    },
                );
            }))
            .map_err(|_| anyhow!("Error senidng remove cluster file callback to CbSink!"))
    }
}

impl Default for ClusterTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Opens an audio file actions dialog for the given audio file view.
///
/// # Arguments
///
/// * `s` - The Cursive to open the dialog with.
/// * `view` - The audio file view to open the dialog for.
fn audio_file_dialog(s: &mut Cursive, audio_file: AudioFile, title: String) {
    let lookup = Button::new("Lookup", |_| {});
    let scan = Button::new("Scan", move |_| {
        if let Err(e) = UiEventService::instance().send(UiEvent::ScanAudioFile(audio_file.clone()))
        {
            error!("Error sending scan audio file event: {e}!");
        }
    });
    let save = Button::new("Save", |_| {});
    let remove = Button::new("Remove", |_| {});
    let layout = LinearLayout::vertical()
        .child(lookup)
        .child(scan)
        .child(save)
        .child(remove);
    let dialog = Dialog::around(layout)
        .title(title.clone())
        //.button("Save", move |s: &mut Cursive| {...})
        .button("Cancel", |s| {
            s.pop_layer();
        });
    s.add_layer(dialog);
}
