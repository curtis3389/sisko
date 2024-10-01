use super::{CbSinkService, UiEventService};
use crate::{
    domain::models::{Album, AudioFile, Metadata, Track},
    ui::{
        events::UiEvent,
        models::{AlbumView, AlbumViewId, AudioFileColumn, MatchState, ALBUM_FILE_TABLE},
    },
};
use anyhow::{anyhow, Result};
use cursive::{
    views::{Button, Dialog, LinearLayout},
    Cursive,
};
use cursive_table_view::TableView;
use log::error;

pub struct AlbumTable {}

impl AlbumTable {
    pub fn new() -> Self {
        Self {}
    }

    pub fn add_album(
        &self,
        album: &Album,
        tracks: &[Track],
        match_states: &Vec<MatchState>,
    ) -> Result<()> {
        let album_id = album.id.clone();
        let album_views = AlbumView::all_for_album(album, tracks, match_states);
        CbSinkService::instance()?
            .send(Box::new(move |s: &mut Cursive| {
                s.call_on_name(
                    ALBUM_FILE_TABLE,
                    |table: &mut TableView<AlbumView, AudioFileColumn>| {
                        if !table
                            .borrow_items()
                            .iter()
                            .any(|item| *item.id.album_id() == album_id)
                        {
                            for view in album_views {
                                table.insert_item(view);
                            }
                        }
                    },
                );
            }))
            .map_err(|_| anyhow!("Error sending add album callback to CbSink!"))
    }

    pub fn open_album_view_dialog(
        &self,
        audio_file: &Option<AudioFile>,
        title: &str,
    ) -> Result<()> {
        let audio_file = audio_file.clone();
        let title = title.to_owned();
        CbSinkService::instance()?
            .send(Box::new(move |s: &mut Cursive| {
                if let Err(e) = album_view_dialog(s, audio_file, title) {
                    error!("Error opening album view dialog: {e}!");
                }
            }))
            .map_err(|_| anyhow!("Error sending open album view dialog callback to CbSink!"))
    }

    // need to match add_album
    pub fn update_audio_file(&self, audio_file: &AudioFile, metadata: &Metadata) -> Result<()> {
        let audio_file = audio_file.clone();
        CbSinkService::instance()?
            .send(Box::new(move |s: &mut Cursive| {
                s.call_on_name(
                    ALBUM_FILE_TABLE,
                    |table: &mut TableView<AlbumView, AudioFileColumn>| {
                        table
                            .borrow_items_mut()
                            .iter_mut()
                            .filter(|item| {
                                audio_file
                                    .track_id
                                    .as_ref()
                                    .map(|id| {
                                        let track_id = match &item.id {
                                            AlbumViewId::Album(_album_id) => None,
                                            AlbumViewId::Track(track_id) => {
                                                Some(track_id.track_id.clone())
                                            }
                                        };
                                        id.album_id == *item.id.album_id()
                                            && (Some(id.track_id.clone()) == track_id
                                                || track_id.is_none())
                                    })
                                    .unwrap_or_default()
                            })
                            .for_each(|item| item.update_for_audio_file(&audio_file));
                    },
                );
            }))
            .map_err(|_| anyhow!("Error sending add album callback to CbSink!"))
    }
}

impl Default for AlbumTable {
    fn default() -> Self {
        Self::new()
    }
}

fn album_view_dialog(s: &mut Cursive, audio_file: Option<AudioFile>, title: String) -> Result<()> {
    let save = Button::new("Save", move |_| {
        if let Some(audio_file) = &audio_file {
            if let Err(e) =
                UiEventService::instance().send(UiEvent::SaveAudioFile(audio_file.clone()))
            {
                error!("Error sending scan audio file event: {e}!");
            }
        }
    });
    let cancel = Button::new("Cancel", |s| {
        s.pop_layer();
    });
    let layout = LinearLayout::vertical().child(save).child(cancel);
    let dialog = Dialog::around(layout).title(title);
    s.add_layer(dialog);
    Ok(())
}
