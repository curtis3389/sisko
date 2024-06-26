use crate::domain::{File, FileType, IFileService, ISiskoService, TagField, Track};
use crate::ui::*;
use cursive::reexports::enumset::enum_set;
use cursive::theme::{ColorStyle, Effect, Style};
use cursive::traits::*;
use cursive::views::{Dialog, EditView, LinearLayout, TextView};
use cursive::{CbSink, Cursive};
use cursive_table_view::TableView;
use std::env;
use std::sync::{Arc, Mutex};
use syrette::ptr::SingletonPtr;
use syrette::{injectable, DIContainer};

/// Represents the UI backed by a cursive callback sink.
pub struct UiWrapper {
    /// A cursive callback sink.
    cb_sink: CbSink,
}

#[injectable(Ui)]
impl UiWrapper {
    /// Returns a new wrapper of the UI.
    ///
    /// # Arguments
    ///
    /// * `cursive` - The cursive root to create a wrapper with.
    pub fn new(cursive: SingletonPtr<dyn ICursive>) -> Self {
        let root = cursive.root().lock().unwrap();
        let cb_sink = root.cb_sink().clone();
        UiWrapper { cb_sink }
    }
}

impl Ui for UiWrapper {
    fn add_cluster_file(&self, track: Arc<Mutex<Track>>) {
        let track_view = TrackView::from(&track);
        self.cb_sink
            .send(Box::new(move |s: &mut Cursive| {
                s.call_on_name(
                    CLUSTER_FILE_TABLE,
                    |table: &mut TableView<TrackView, TrackColumn>| {
                        table.insert_item(track_view);
                    },
                );
            }))
            .unwrap();
    }

    fn open_directory_dialog(&self) {
        self.cb_sink
            .send(Box::new(move |s: &mut Cursive| {
                new_file_dialog(
                    s,
                    FileDialogType::Directory,
                    |s: &mut Cursive, f: Arc<File>| {
                        let container = s.user_data().unwrap() as &mut DIContainer;
                        let sisko_service = container
                            .get::<dyn ISiskoService>()
                            .unwrap()
                            .transient()
                            .unwrap();
                        sisko_service.add_folder(f);
                    },
                );
            }))
            .unwrap();
    }

    fn open_tag_field_dialog(&self, field: &TagFieldView) {
        let field = field.clone();
        self.cb_sink
            .send(Box::new(move |s: &mut Cursive| {
                tag_field_dialog(s, &field);
            }))
            .unwrap();
    }

    fn set_metadata_table(&self, track: &Arc<Mutex<Track>>) {
        let arc = track;
        let track = track.lock().unwrap();
        let items: Vec<TagFieldView> = track
            .tags
            .iter()
            .map(|tag| {
                tag.fields
                    .iter()
                    .map(|f| TagFieldView::new(arc, &tag.tag_type, &f.tag_field_type()))
            })
            .flatten()
            .collect();
        self.cb_sink
            .send(Box::new(move |s: &mut Cursive| {
                s.call_on_name(
                    METADATA_TABLE,
                    |table: &mut TableView<TagFieldView, TagFieldColumn>| {
                        table.set_items(items);
                    },
                );
            }))
            .unwrap();
    }
}

/// Opens a new file/folder selection dialog of the given type and calls the
/// given callback with the chosen file/folder.
///
/// # Arguments
///
/// * `dialog_type` - The type of the dialog to open.
/// * `on_choose` - The callback to call when user makes their choice.
fn new_file_dialog<F>(s: &mut Cursive, dialog_type: FileDialogType, on_choose: F)
where
    F: 'static + Fn(&mut Cursive, Arc<File>) + Clone,
{
    let current_directory = env::current_dir().unwrap();
    let container = s.user_data().unwrap() as &mut DIContainer;
    let file_service = container
        .get::<dyn IFileService>()
        .unwrap()
        .singleton()
        .unwrap();
    let files: Vec<FileView> = file_service
        .get_files_in_dir(&current_directory, dialog_type)
        .iter()
        .map(|f| FileView::from(f))
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
            let selected_file = s
                .call_on_name(
                    FILE_TABLE,
                    |table_view: &mut TableView<FileView, FileColumn>| {
                        let item = table_view.borrow_item(index).unwrap();
                        item.clone()
                    },
                )
                .unwrap();
            let selected_file = selected_file.file;
            if dialog_type == FileDialogType::AudioFile
                && selected_file.file_type != Some(FileType::Directory)
            {
                on_choose_copy(s, selected_file);
                s.pop_layer();
            } else {
                if selected_file.file_type == Some(FileType::Directory) {
                    let container = s.user_data().unwrap() as &mut DIContainer;
                    let file_service = container
                        .get::<dyn IFileService>()
                        .unwrap()
                        .singleton()
                        .unwrap();
                    let files: Vec<FileView> = file_service
                        .get_files_in_dir(&selected_file.path, dialog_type)
                        .iter()
                        .map(|f| FileView::from(f))
                        .collect();

                    s.call_on_name(
                        FILE_TABLE,
                        |table_view: &mut TableView<FileView, FileColumn>| {
                            table_view.set_items(files);
                        },
                    );
                }
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
            let file = s
                .call_on_name(
                    FILE_TABLE,
                    |table_view: &mut TableView<FileView, FileColumn>| {
                        table_view
                            .borrow_item(table_view.item().unwrap())
                            .unwrap()
                            .clone()
                    },
                )
                .unwrap();
            on_choose(s, file.file);
            s.pop_layer();
        })
        .button("Cancel", |s| {
            s.pop_layer();
        });

    s.add_layer(dialog);
}

fn tag_field_dialog(s: &mut Cursive, view: &TagFieldView) {
    let track = view.track.clone();
    let tag_type = view.tag_type.clone();
    let field = view.get_field();
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
            let new_field_value = s
                .call_on_name(NEW_FIELD_VALUE, |edit_view: &mut EditView| {
                    edit_view.get_content().as_ref().clone()
                })
                .unwrap();
            let field = field.clone();
            let field = match field {
                TagField::Binary(_, _, _new_field_value) => todo!("handle selected file"),
                TagField::Text(tag_field_type, value, _) => {
                    TagField::Text(tag_field_type, value, Some(new_field_value))
                }
                TagField::Unknown(_, _) => field,
            };
            let mut track = track.lock().unwrap();
            track.update_tag_field(&tag_type, field);
            s.pop_layer();
        })
        .button("Cancel", |s| {
            s.pop_layer();
        });
    s.add_layer(dialog);
}
