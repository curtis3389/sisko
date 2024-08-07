use crate::domain::{Album, AudioFile, File, FileService, FileType, TagField};
use crate::ui::*;
use anyhow::{anyhow, Result};
use cursive::reexports::enumset::enum_set;
use cursive::theme::{ColorStyle, Effect, Style};
use cursive::traits::*;
use cursive::views::{Button, Dialog, EditView, LinearLayout, ScrollView, TextView};
use cursive::Cursive;
use cursive_table_view::TableView;
use log::error;
use std::env;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

/// Represents the UI backed by a cursive callback sink.
pub struct UiWrapper {}

impl UiWrapper {
    pub fn instance() -> &'static Self {
        static INSTANCE: OnceLock<UiWrapper> = OnceLock::new();
        INSTANCE.get_or_init(Self::new)
    }

    /// Returns a new wrapper of the UI.
    pub fn new() -> Self {
        Self {}
    }

    pub fn add_album(&self, album: Arc<Mutex<Album>>) -> Result<()> {
        let album_id = {
            album
                .lock()
                .map_err(|_| anyhow!("Error locking album mutex!"))?
                .id
                .clone()
        };
        let album_views = AlbumView::for_album(&album)?;
        CbSinkService::instance()?
            .send(Box::new(move |s: &mut Cursive| {
                s.call_on_name(
                    ALBUM_FILE_TABLE,
                    |table: &mut TableView<AlbumView, AudioFileColumn>| {
                        if !table.borrow_items().iter().any(|item| item.id == album_id) {
                            for view in album_views {
                                table.insert_item(view);
                            }
                        }
                    },
                );
            }))
            .map_err(|_| anyhow!("Error sending add album callback to CbSink!"))?;
        Ok(())
    }

    pub fn add_cluster_file(&self, audio_file: Arc<Mutex<AudioFile>>) -> Result<()> {
        let audio_file_view = AudioFileView::try_from(&audio_file)?;
        CbSinkService::instance()?
            .send(Box::new(move |s: &mut Cursive| {
                s.call_on_name(
                    CLUSTER_FILE_TABLE,
                    |table: &mut TableView<AudioFileView, AudioFileColumn>| {
                        if !table
                            .borrow_items()
                            .iter()
                            .any(|i| i.path == audio_file_view.path)
                        {
                            table.insert_item(audio_file_view.clone());
                            if table.len() == 1 {
                                if let Err(e) = UiEventService::instance()
                                    .send(UiEvent::SelectClusterFile(audio_file_view))
                                {
                                    error!("Error sending select cluster file event: {e}!");
                                    // TODO: log error
                                };
                            }
                        }
                    },
                );
            }))
            .map_err(|_| anyhow!("Error sending add cluster file callback to CbSink!"))
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

    pub fn open_tag_field_dialog(&self, field: &TagFieldView) -> Result<()> {
        let field = field.clone();
        CbSinkService::instance()?
            .send(Box::new(move |s: &mut Cursive| {
                if let Err(e) = tag_field_dialog(s, &field) {
                    error!("Error opening tag field dialog: {e}!");
                }
            }))
            .map_err(|_| anyhow!("Error sending open tag field dialog callback to CbSink!"))
    }

    pub fn open_audio_file_dialog(&self, audio_file: &AudioFileView) -> Result<()> {
        let audio_file = audio_file.clone();
        CbSinkService::instance()?
            .send(Box::new(move |s: &mut Cursive| {
                audio_file_dialog(s, &audio_file);
            }))
            .map_err(|_| anyhow!("Error sending open audio file dialog callback to CbSink!"))
    }

    pub fn remove_cluster_file(&self, audio_file: &Arc<Mutex<AudioFile>>) -> Result<()> {
        let path = audio_file
            .lock()
            .map_err(|_| anyhow!("Error locking audio file mutex!"))?
            .file
            .absolute_path
            .clone();
        CbSinkService::instance()?
            .send(Box::new(move |s: &mut Cursive| {
                s.call_on_name(
                    CLUSTER_FILE_TABLE,
                    |table: &mut TableView<AudioFileView, AudioFileColumn>| {
                        if let Some((index, _)) = table
                            .borrow_items()
                            .iter()
                            .enumerate()
                            .find(|(_, item)| item.path == path)
                        {
                            table.remove_item(index);
                        }
                    },
                );
            }))
            .map_err(|_| anyhow!("Error senidng remove cluster file callback to CbSink!"))
    }

    pub fn set_metadata_table(&self, audio_file: &Arc<Mutex<AudioFile>>) -> Result<()> {
        let arc = audio_file;
        let audio_file = audio_file
            .lock()
            .map_err(|_| anyhow!("Failed to lock audio file mutex!"))?;
        let items: Vec<TagFieldView> = audio_file
            .tags
            .iter()
            .flat_map(|tag| {
                tag.fields
                    .iter()
                    .map(|f| TagFieldView::new(arc, &tag.tag_type, &f.tag_field_type()))
            })
            .collect();
        CbSinkService::instance()?
            .send(Box::new(move |s: &mut Cursive| {
                s.call_on_name(
                    METADATA_TABLE,
                    |table: &mut TableView<TagFieldView, TagFieldColumn>| {
                        table.set_items(items);
                    },
                );
            }))
            .map_err(|_| anyhow!("Error sending set metadata table callback to CbSink!"))
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

/// Opens a tag field edit dialog for the given tag field view.
///
/// # Arguments
///
/// * `s` - The Cursive to open the dialog with.
/// * `view` - The tag field view to open the dialog for.
fn tag_field_dialog(s: &mut Cursive, view: &TagFieldView) -> Result<()> {
    let audio_file = view.audio_file.clone();
    let tag_type = view.tag_type.clone();
    let field = view.get_field()?;
    let title = field.display_name();
    let name = LinearLayout::horizontal()
        .child(TextView::new(String::from("Tag: ")).style(Style {
            effects: enum_set!(Effect::Bold),
            color: ColorStyle::inherit_parent(),
        }))
        .child(TextView::new(title.clone()));
    let value = LinearLayout::horizontal()
        .child(
            TextView::new(String::from("Original Value: ")).style(Style {
                effects: enum_set!(Effect::Bold),
                color: ColorStyle::inherit_parent(),
            }),
        )
        .child(TextView::new(field.display_value()));
    let mut new_value =
        LinearLayout::horizontal().child(TextView::new(String::from("New Value: ")).style(Style {
            effects: enum_set!(Effect::Bold),
            color: ColorStyle::inherit_parent(),
        }));
    match &field {
        TagField::Binary(_, _, _new_field_value) => todo!("add file selector"),
        TagField::Text(_, _, new_field_value) => new_value.add_child(
            EditView::new()
                .content(new_field_value.clone().unwrap_or(String::new()))
                .with_name(NEW_FIELD_VALUE)
                .fixed_width(32),
        ),
        TagField::Unknown(_, _) => new_value.add_child(TextView::new(String::new())),
    }
    let layout = LinearLayout::vertical()
        .child(name)
        .child(value)
        .child(new_value);
    let dialog = Dialog::around(layout)
        .title(title)
        .button("Save", move |s: &mut Cursive| {
            if let Err(e) = || -> Result<()> {
                let new_field_value = s
                    .call_on_name(NEW_FIELD_VALUE, |edit_view: &mut EditView| {
                        edit_view.get_content().as_ref().clone()
                    })
                    .ok_or_else(|| {
                        anyhow!(
                            "Failed to call on save lambda to get edit field content in {}!",
                            NEW_FIELD_VALUE
                        )
                    })?;
                let field = field.clone();
                let field = match field {
                    TagField::Binary(_, _, _new_field_value) => todo!("handle selected file"),
                    TagField::Text(tag_field_type, value, _) => {
                        TagField::Text(tag_field_type, value, Some(new_field_value))
                    }
                    TagField::Unknown(_, _) => field,
                };
                let mut audio_file = audio_file
                    .lock()
                    .map_err(|_| anyhow!("Failed to lock audio file mutex!"))?;
                audio_file.update_tag_field(&tag_type, field)?;
                s.pop_layer();
                Ok(())
            }() {
                error!("Error saving new field value: {e}!");
            }
        })
        .button("Cancel", |s| {
            s.pop_layer();
        });
    s.add_layer(dialog);
    Ok(())
}

/// Opens an audio file actions dialog for the given audio file view.
///
/// # Arguments
///
/// * `s` - The Cursive to open the dialog with.
/// * `view` - The audio file view to open the dialog for.
fn audio_file_dialog(s: &mut Cursive, view: &AudioFileView) {
    let audio_file = view.audio_file.clone();
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
        .title(view.title.clone())
        //.button("Save", move |s: &mut Cursive| {...})
        .button("Cancel", |s| {
            s.pop_layer();
        });
    s.add_layer(dialog);
}

impl Default for UiWrapper {
    fn default() -> Self {
        Self::new()
    }
}
