use crate::file::File;
use crate::file_column::FileColumn;
use crate::file_dialog_type::FileDialogType;
use crate::file_service::IFileService;
use crate::file_type::FileType;
use crate::icursive::ICursive;
use crate::sisko_service::ISiskoService;
use crate::tag::Tag;
use crate::tag_column::TagColumn;
use crate::track::Track;
use crate::track_column::TrackColumn;
use crate::ui_element::*;
use cursive::event::{Event, Key};
use cursive::traits::*;
use cursive::views::{Button, Dialog, HideableView, LinearLayout, NamedView, Panel, ResizedView};
use cursive::{menu, CbSink, Cursive, CursiveRunnable};
use cursive_table_view::TableView;
use std::env;
use syrette::ptr::SingletonPtr;
use syrette::{injectable, DIContainer};

/// Represents the UI.
pub struct Ui {
    container: DIContainer,
}

impl Ui {
    /// Returns a new UI.
    ///
    /// # Arguments
    ///
    /// * `root` - The cursive root to create the UI with.
    pub fn new(container: DIContainer) -> Ui {
        let cursive = container
            .get::<dyn ICursive>()
            .unwrap()
            .singleton()
            .unwrap();
        let mut root = cursive.root().lock().unwrap();
        Self::add_hotkeys(&mut root);
        Self::setup_menubar(&mut root);
        Self::add_widgets(&mut root);
        Ui { container }
    }

    /// Runs the UI.
    pub fn run(self) {
        let cursive = self
            .container
            .get::<dyn ICursive>()
            .unwrap()
            .singleton()
            .unwrap();
        let mut root = cursive.root().lock().unwrap();
        root.set_user_data(self.container);
        root.run();
    }

    /// Adds the hotkeys to the UI.
    ///
    /// # Arguments
    ///
    /// * `root` - The root to add the hotkeys to.
    fn add_hotkeys(root: &mut CursiveRunnable) {
        root.add_global_callback(Key::Esc, |s| s.select_menubar());
        root.add_global_callback(Event::CtrlChar('b'), |s| {
            s.call_on_name(
                HIDEABLE_BOTTOM_PANEL,
                |hideable: &mut HideableView<
                    Panel<ResizedView<NamedView<TableView<Tag, TagColumn>>>>,
                >| {
                    hideable.set_visible(!hideable.is_visible());
                },
            );
        });
        root.add_global_callback(Event::CtrlChar('l'), |s| {
            s.call_on_name(
                HIDEABLE_LEFT_PANEL,
                |hideable: &mut HideableView<
                    Panel<ResizedView<NamedView<TableView<Track, TrackColumn>>>>,
                >| {
                    hideable.set_visible(!hideable.is_visible());
                },
            );
        });
        root.add_global_callback(Event::CtrlChar('r'), |s| {
            s.call_on_name(
                HIDEABLE_RIGHT_PANEL,
                |hideable: &mut HideableView<
                    Panel<ResizedView<NamedView<TableView<Track, TrackColumn>>>>,
                >| {
                    hideable.set_visible(!hideable.is_visible());
                },
            );
        });
    }

    /// Configures the menubar in the UI.
    ///
    /// # Arguments
    ///
    /// * `root` - The root to configure the menubar on.
    fn setup_menubar(root: &mut CursiveRunnable) {
        root.menubar()
            .add_subtree("File", menu::Tree::new().leaf("Quit", Cursive::quit))
            .add_subtree("Edit", menu::Tree::new())
            .add_subtree("View", menu::Tree::new())
            .add_subtree("Options", menu::Tree::new())
            .add_subtree("Tools", menu::Tree::new())
            .add_subtree("Help", menu::Tree::new());
        root.set_autohide_menu(false);
    }

    /// Adds the widgets to the UI.
    ///
    /// # Arguments
    ///
    /// * `root` - The root to add the widgets to.
    fn add_widgets(root: &mut CursiveRunnable) {
        let cluster_file_table = TableView::<Track, TrackColumn>::new()
            .column(TrackColumn::Title, TrackColumn::Title.as_str(), |c| c)
            .column(TrackColumn::Artist, TrackColumn::Artist.as_str(), |c| c)
            .column(TrackColumn::Length, TrackColumn::Length.as_str(), |c| c)
            .on_select(|s: &mut Cursive, _row: usize, _index: usize| {
                /*let selected_track = s
                .call_on_name(
                    CLUSTER_FILE_TABLE,
                    |table_view: &mut TableView<Track, TrackColumn>| {
                        let item = table_view.borrow_item(index).unwrap();
                        item.clone()
                    },
                )
                .unwrap();*/
                let tags: Vec<Tag> = vec![];
                /*let tags = selected_track
                .tag
                .fields()
                .iter()
                .map(|field| Tag {
                    tag: field.display_name(),
                    original_value: field.display_value().unwrap_or("<empty>".to_string()),
                    new_value: "".to_string(),
                })
                .collect();*/
                s.call_on_name(METADATA_TABLE, |table: &mut TableView<Tag, TagColumn>| {
                    table.set_items(tags);
                });
            })
            .with_name(CLUSTER_FILE_TABLE);

        let album_file_table = TableView::<Track, TrackColumn>::new()
            .column(TrackColumn::Title, TrackColumn::Title.as_str(), |c| c)
            .column(TrackColumn::Artist, TrackColumn::Artist.as_str(), |c| c)
            .column(TrackColumn::Length, TrackColumn::Length.as_str(), |c| c)
            .with_name(ALBUM_FILE_TABLE);

        let metadata_table = TableView::<Tag, TagColumn>::new()
            .column(TagColumn::Tag, TagColumn::Tag.as_str(), |c| c)
            .column(
                TagColumn::OriginalValue,
                TagColumn::OriginalValue.as_str(),
                |c| c,
            )
            .column(TagColumn::NewValue, TagColumn::NewValue.as_str(), |c| c)
            .with_name(METADATA_TABLE);

        let hideable_bottom_panel = HideableView::new(Panel::new(metadata_table.full_screen()))
            .with_name(HIDEABLE_BOTTOM_PANEL);
        let hideable_left_panel = HideableView::new(Panel::new(cluster_file_table.full_screen()))
            .with_name(HIDEABLE_LEFT_PANEL);
        let hideable_right_panel = HideableView::new(Panel::new(album_file_table.full_screen()))
            .with_name(HIDEABLE_RIGHT_PANEL);

        let file_panes = LinearLayout::horizontal()
            .child(hideable_left_panel)
            .child(hideable_right_panel);
        let bottom_buttons = LinearLayout::horizontal()
            .child(Button::new("Add Folder", |cursive| {
                let container = cursive.user_data().unwrap() as &mut DIContainer;
                let ui = container.get::<dyn IUi>().unwrap().singleton().unwrap();
                ui.open_directory_dialog();
            }))
            .child(Button::new("Add Files", do_nothing))
            .child(Button::new("Cluster", do_nothing))
            .child(Button::new("Lookup", do_nothing))
            .child(Button::new("Scan", do_nothing))
            .child(Button::new("Save", do_nothing))
            .child(Button::new("Info", do_nothing))
            .child(Button::new("Remove", do_nothing))
            .child(Button::new("Lookup CD", do_nothing));
        let top_level_layout = LinearLayout::vertical()
            .child(file_panes)
            .child(hideable_bottom_panel)
            .child(bottom_buttons);
        root.add_fullscreen_layer(top_level_layout);
    }
}

/// Represents the UI.
pub trait IUi {
    /// Adds the given track to the cluster file table.
    ///
    /// # Arguments
    ///
    /// * `track` - The track to add to the cluster file table.
    fn add_cluster_file(&self, track: Track);

    /// Opens the add directory dialog.
    fn open_directory_dialog(&self);
}

/// Represents the UI backed by a cursive callback sink.
pub struct UiWrapper {
    /// A cursive callback sink.
    cb_sink: CbSink,
}

#[injectable(IUi)]
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

impl IUi for UiWrapper {
    fn add_cluster_file(&self, track: Track) {
        self.cb_sink
            .send(Box::new(move |s: &mut Cursive| {
                s.call_on_name(
                    CLUSTER_FILE_TABLE,
                    |table: &mut TableView<Track, TrackColumn>| {
                        table.insert_item(track);
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
}

/// Does nothing.
fn do_nothing(_: &mut Cursive) {}

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
    let files = file_service.get_files_in_dir(&current_directory, dialog_type);

    let on_choose_copy = on_choose.clone();
    let file_table = TableView::<File, FileColumn>::new()
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
                    |table_view: &mut TableView<File, FileColumn>| {
                        let item = table_view.borrow_item(index).unwrap();
                        item.clone()
                    },
                )
                .unwrap();

            if dialog_type == FileDialogType::AudioFile
                && selected_file.file_type != Some(FileType::Directory)
            {
                on_choose_copy(s, selected_file);
                s.pop_layer();
            } else {
                let container = s.user_data().unwrap() as &mut DIContainer;
                let file_service = container
                    .get::<dyn IFileService>()
                    .unwrap()
                    .singleton()
                    .unwrap();
                let files = file_service.get_files_in_dir(&selected_file.path, dialog_type);

                s.call_on_name(
                    FILE_TABLE,
                    |table_view: &mut TableView<File, FileColumn>| {
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
                    |table_view: &mut TableView<File, FileColumn>| {
                        table_view
                            .borrow_item(table_view.item().unwrap())
                            .unwrap()
                            .clone()
                    },
                )
                .unwrap();
            on_choose(s, file);
            s.pop_layer();
        })
        .button("Cancel", |s| {
            s.pop_layer();
        });

    s.add_layer(dialog);
}
