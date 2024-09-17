use crate::ui::models::{
    AlbumView, AudioFileColumn, AudioFileView, FileColumn, FileView, TagFieldColumn, TagFieldView,
    ALBUM_FILE_TABLE, CLUSTER_FILE_TABLE, FILE_TABLE, METADATA_TABLE,
};
use anyhow::{anyhow, Result};
use cursive::Cursive;
use cursive_table_view::TableView;

pub trait CursiveExtensions {
    fn clone_album_view(&mut self, index: usize) -> Result<AlbumView>;
    fn clone_audio_file_view(&mut self, index: usize) -> Result<AudioFileView>;
    fn clone_file_view(&mut self, index: usize) -> Result<FileView>;
    fn clone_tag_field_view(&mut self, index: usize) -> Result<TagFieldView>;
}

impl CursiveExtensions for Cursive {
    fn clone_album_view(&mut self, index: usize) -> Result<AlbumView> {
        self.call_on_name(
            ALBUM_FILE_TABLE,
            |table_view: &mut TableView<AlbumView, AudioFileColumn>| -> Result<AlbumView> {
                let item = table_view.borrow_item(index).ok_or_else(|| {
                    anyhow!(
                        "Failed to find item at index {} in {}!",
                        index,
                        ALBUM_FILE_TABLE
                    )
                })?;
                Ok(item.clone())
            },
        )
        .ok_or_else(|| anyhow!("Failed to call lambda on {}!", ALBUM_FILE_TABLE))
        .and_then(|r| r)
    }

    fn clone_audio_file_view(&mut self, index: usize) -> Result<AudioFileView> {
        self.call_on_name(
            CLUSTER_FILE_TABLE,
            |table_view: &mut TableView<AudioFileView, AudioFileColumn>| -> Result<AudioFileView> {
                let item = table_view.borrow_item(index).ok_or_else(|| {
                    anyhow!(
                        "Failed to find item at index {} in {}!",
                        index,
                        CLUSTER_FILE_TABLE
                    )
                })?;
                Ok(item.clone())
            },
        )
        .ok_or_else(|| anyhow!("Failed to call lambda on {}!", CLUSTER_FILE_TABLE))
        .and_then(|r| r)
    }

    fn clone_file_view(&mut self, index: usize) -> Result<FileView> {
        self.call_on_name(
            FILE_TABLE,
            |table_view: &mut TableView<FileView, FileColumn>| -> Result<FileView> {
                let item = table_view.borrow_item(index).ok_or_else(|| {
                    anyhow!("Failed to find item at index {} in {}!", index, FILE_TABLE)
                })?;
                Ok(item.clone())
            },
        )
        .ok_or_else(|| anyhow!("Failed to call lambda on {}!", FILE_TABLE))
        .and_then(|r| r)
    }

    fn clone_tag_field_view(&mut self, index: usize) -> Result<TagFieldView> {
        self.call_on_name(
            METADATA_TABLE,
            |table_view: &mut TableView<TagFieldView, TagFieldColumn>| -> Result<TagFieldView> {
                let item = table_view.borrow_item(index).ok_or_else(|| {
                    anyhow!(
                        "Failed to find item at index {} in {}!",
                        index,
                        METADATA_TABLE
                    )
                })?;
                Ok(item.clone())
            },
        )
        .ok_or_else(|| anyhow!("Failed to call lambda on {}!", METADATA_TABLE))
        .and_then(|r| r)
    }
}
