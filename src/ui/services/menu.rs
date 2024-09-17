use super::{CbSinkService, UiEventService};
use crate::{
    infrastructure::{
        file::{File, FileService, FileType},
        CursiveExtensions,
    },
    ui::{
        events::UiEvent,
        models::{FileColumn, FileDialogType, FileView, FILE_TABLE},
    },
};
use anyhow::{anyhow, Result};
use cursive::{
    traits::Nameable,
    view::Resizable,
    views::{Dialog, ScrollView, TextView},
    Cursive,
};
use cursive_table_view::TableView;
use log::error;
use std::{env, path::PathBuf, sync::Arc};

pub struct Menu {}

impl Menu {
    pub fn new() -> Self {
        Self {}
    }

    pub fn open_directory_dialog(&self) -> Result<()> {
        CbSinkService::instance()?
            .send(Box::new(move |s: &mut Cursive| {
                new_file_dialog(
                    s,
                    FileDialogType::Directory,
                    |_: &mut Cursive, f: Arc<File>| {
                        if let Err(e) = UiEventService::instance().send(UiEvent::FolderSelected(f))
                        {
                            error!("Error sending folder selected event: {e}!");
                        };
                    },
                );
            }))
            .map_err(|_| anyhow!("Error sending open directory dialog callback to CbSink!"))
    }

    pub fn open_file_dialog(&self) -> Result<()> {
        CbSinkService::instance()?
            .send(Box::new(move |s: &mut Cursive| {
                new_file_dialog(
                    s,
                    FileDialogType::AudioFile,
                    |_: &mut Cursive, f: Arc<File>| {
                        if let Err(e) = UiEventService::instance().send(UiEvent::FileSelected(f)) {
                            error!("Error sending file selected event: {e}!");
                        };
                    },
                );
            }))
            .map_err(|_| anyhow!("Error sending open file dialog callback to CbSink!"))
    }

    pub fn open_logs(&self, logs: &str) -> Result<()> {
        let logs = logs.to_owned();
        CbSinkService::instance()?
            .send(Box::new(move |s: &mut Cursive| {
                new_logs_dialog(s, &logs);
            }))
            .map_err(|_| anyhow!("Error sending open logs callback to CbSink!"))
    }
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}

fn new_logs_dialog(s: &mut Cursive, logs: &String) {
    let text_view = TextView::new(logs);
    let dialog = Dialog::around(ScrollView::new(text_view))
        .title("Logs")
        .button("Close", |s| {
            s.pop_layer();
        });

    s.add_layer(dialog);
}

/// Opens a new file/folder selection dialog of the given type and calls the
/// given callback with the chosen file/folder.
///
/// # Arguments
///
/// * `s` - The Cursive to open the dialog with.
/// * `dialog_type` - The type of the dialog to open.
/// * `on_choose` - The callback to call when user makes their choice.
fn new_file_dialog<F>(s: &mut Cursive, dialog_type: FileDialogType, on_choose: F)
where
    F: 'static + Fn(&mut Cursive, Arc<File>) + Clone,
{
    let current_directory = match env::current_dir() {
        Ok(dir) => dir,
        _ => PathBuf::default(),
    };
    let files: Vec<FileView> = FileService::instance()
        .get_files_in_dir(&current_directory, dialog_type)
        .unwrap_or_default() // TODO: log warning and provide error to user
        .iter()
        .map(FileView::from)
        .collect();

    let on_choose_copy = on_choose.clone();
    let file_table = TableView::<FileView, FileColumn>::new()
        .column(FileColumn::Name, FileColumn::Name.as_str(), |c| c)
        .column(FileColumn::Size, FileColumn::Size.as_str(), |c| c)
        .column(FileColumn::Type, FileColumn::Type.as_str(), |c| c)
        .column(
            FileColumn::DateModified,
            FileColumn::DateModified.as_str(),
            |c| c,
        )
        .on_submit(move |s: &mut Cursive, _row: usize, index: usize| {
            if let Err(e) = (|| -> Result<()> {
                let selected_file = s.clone_file_view(index)?;
                let selected_file = selected_file.file;
                if dialog_type == FileDialogType::AudioFile
                    && selected_file.file_type != Some(FileType::Directory)
                {
                    on_choose_copy(s, selected_file);
                    s.pop_layer();
                } else if selected_file.file_type == Some(FileType::Directory) {
                    let files: Vec<FileView> = FileService::instance()
                        .get_files_in_dir(&selected_file.absolute_path, dialog_type)?
                        .iter()
                        .map(FileView::from)
                        .collect();

                    s.call_on_name(
                        FILE_TABLE,
                        |table_view: &mut TableView<FileView, FileColumn>| {
                            table_view.set_items(files);
                        },
                    )
                    .ok_or_else(|| anyhow!("Failed to call set items on file table!"))?;
                }
                Ok(())
            })() {
                error!("Error handling on submit event on file table: {e}!");
            }
        })
        .items(files)
        .with_name(FILE_TABLE)
        .full_screen();

    let title = match dialog_type {
        FileDialogType::AudioFile => "Find Audio File",
        FileDialogType::Directory => "Find Directory",
    };
    let dialog = Dialog::around(file_table)
        .title(title)
        .button("Choose", move |s: &mut Cursive| {
            if let Ok(file) = s
                .call_on_name(
                    FILE_TABLE,
                    |table_view: &mut TableView<FileView, FileColumn>| -> Result<FileView> {
                        let index = table_view.item().ok_or(anyhow!(
                            "Error getting currently selected item in {} when there is none!",
                            FILE_TABLE
                        ))?;
                        Ok(table_view
                            .borrow_item(index)
                            .ok_or(anyhow!(
                                "Error getting currently selected item in {} by index {}!",
                                FILE_TABLE,
                                index
                            ))?
                            .clone())
                    },
                )
                .ok_or_else(|| {
                    anyhow!("Failed to call on choose lambda to get selected item in {FILE_TABLE}!")
                })
                .and_then(|r| r)
            {
                on_choose(s, file.file);
                s.pop_layer();
            }
        })
        .button("Cancel", |s| {
            s.pop_layer();
        });

    s.add_layer(dialog);
}
