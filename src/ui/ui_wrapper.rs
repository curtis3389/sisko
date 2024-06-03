use crate::domain::{File, FileType, IFileService, ISiskoService, TagField, Track};
use crate::ui::*;
use cursive::traits::*;
use cursive::views::Dialog;
use cursive::{CbSink, Cursive};
use cursive_table_view::TableView;
use std::env;
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
    fn add_cluster_file(&self, track: Track) {
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
                new_file_dialog(s, FileDialogType::Directory, |s: &mut Cursive, f: File| {
                    let container = s.user_data().unwrap() as &mut DIContainer;
                    let sisko_service = container
                        .get::<dyn ISiskoService>()
                        .unwrap()
                        .transient()
                        .unwrap();
                    sisko_service.add_folder(f);
                });
            }))
            .unwrap();
    }

    fn set_metadata_table(&self, fields: &Vec<TagField>) {
        let items: Vec<TagFieldView> = fields.iter().map(|f| TagFieldView::from(f)).collect();
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
    F: 'static + Fn(&mut Cursive, File) + Clone,
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
            let container = s.user_data().unwrap() as &mut DIContainer;
            let file_service = container
                .get::<dyn IFileService>()
                .unwrap()
                .singleton()
                .unwrap();
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
            let file = file.file;
            on_choose(s, file);
            s.pop_layer();
        })
        .button("Cancel", |s| {
            s.pop_layer();
        });

    s.add_layer(dialog);
}
