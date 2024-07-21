use super::{
    AlbumView, CbSinkService, TagFieldColumn, TagFieldView, TrackColumn, TrackView, UiEvent,
    UiEventService, ALBUM_FILE_TABLE, CLUSTER_FILE_TABLE, HIDEABLE_BOTTOM_PANEL,
    HIDEABLE_LEFT_PANEL, HIDEABLE_RIGHT_PANEL, METADATA_TABLE,
};
use cursive::event::{Event, Key};
use cursive::traits::*;
use cursive::views::{Button, HideableView, LinearLayout, NamedView, Panel, ResizedView};
use cursive::{menu, Cursive, CursiveRunnable};
use cursive_table_view::TableView;
use std::sync::Mutex;

type TablePanel<Row, Column> = HideableView<Panel<ResizedView<NamedView<TableView<Row, Column>>>>>;

/// Represents a wrapper for the default cursive root.
pub struct CursiveWrapper {
    /// The default cursive root.
    root: Mutex<CursiveRunnable>,
}

impl CursiveWrapper {
    /// Returns a new wrapper for the default cursive root.
    pub fn new() -> Self {
        let mut root = cursive::default();

        Self::add_hotkeys(&mut root);
        Self::setup_menubar(&mut root);
        Self::add_widgets(&mut root);

        CbSinkService::set_instance(root.cb_sink().clone());

        CursiveWrapper {
            root: Mutex::new(root),
        }
    }

    /// Adds the hotkeys to the UI.
    ///
    /// # Arguments
    ///
    /// * `root` - The root to add the hotkeys to.
    fn add_hotkeys(root: &mut CursiveRunnable) {
        // TODO: update this to s.send_event(UiEvent::GlobalKeyPress(...))
        root.add_global_callback(Key::Esc, |s| s.select_menubar());
        root.add_global_callback(Event::CtrlChar('b'), |s| {
            s.call_on_name(
                HIDEABLE_BOTTOM_PANEL,
                |hideable: &mut TablePanel<TagFieldView, TagFieldColumn>| {
                    hideable.set_visible(!hideable.is_visible());
                },
            );
        });
        root.add_global_callback(Event::CtrlChar('l'), |s| {
            s.call_on_name(
                HIDEABLE_LEFT_PANEL,
                |hideable: &mut TablePanel<TrackView, TrackColumn>| {
                    hideable.set_visible(!hideable.is_visible());
                },
            );
        });
        root.add_global_callback(Event::CtrlChar('r'), |s| {
            s.call_on_name(
                HIDEABLE_RIGHT_PANEL,
                |hideable: &mut TablePanel<TrackView, TrackColumn>| {
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
            .add_subtree(
                "View",
                menu::Tree::new().leaf("Logs", |_| {
                    UiEventService::instance().send(UiEvent::OpenLogs);
                }),
            )
            .add_subtree("Options", menu::Tree::new())
            .add_subtree("Tools", menu::Tree::new())
            .add_subtree("Help", menu::Tree::new());
        root.set_autohide_menu(false);
    }

    /// Adds the widgets to the UI.p
    ///
    /// # Arguments
    ///
    /// * `root` - The root to add the widgets to.
    fn add_widgets(root: &mut CursiveRunnable) {
        let cluster_file_table = TableView::<TrackView, TrackColumn>::new()
            .column(TrackColumn::Title, TrackColumn::Title.as_str(), |c| c)
            .column(TrackColumn::Artist, TrackColumn::Artist.as_str(), |c| c)
            .column(TrackColumn::Length, TrackColumn::Length.as_str(), |c| c)
            .on_select(|s: &mut Cursive, _row: usize, index: usize| {
                let selected_track = s
                    .call_on_name(
                        CLUSTER_FILE_TABLE,
                        |table_view: &mut TableView<TrackView, TrackColumn>| {
                            let item = table_view.borrow_item(index).unwrap();
                            item.clone()
                        },
                    )
                    .unwrap();
                UiEventService::instance().send(UiEvent::SelectClusterFile(selected_track));
            })
            .on_submit(|s: &mut Cursive, _row: usize, index: usize| {
                let selected_track = s
                    .call_on_name(
                        CLUSTER_FILE_TABLE,
                        |table_view: &mut TableView<TrackView, TrackColumn>| {
                            let item = table_view.borrow_item(index).unwrap();
                            item.clone()
                        },
                    )
                    .unwrap();
                UiEventService::instance().send(UiEvent::SubmitClusterFile(selected_track));
            })
            .with_name(CLUSTER_FILE_TABLE);

        let album_file_table = TableView::<AlbumView, TrackColumn>::new()
            .column(TrackColumn::Title, TrackColumn::Title.as_str(), |c| c)
            .column(TrackColumn::Artist, TrackColumn::Artist.as_str(), |c| c)
            .column(TrackColumn::Length, TrackColumn::Length.as_str(), |c| c)
            .with_name(ALBUM_FILE_TABLE);

        let metadata_table = TableView::<TagFieldView, TagFieldColumn>::new()
            .column(TagFieldColumn::Tag, TagFieldColumn::Tag.as_str(), |c| c)
            .column(
                TagFieldColumn::OriginalValue,
                TagFieldColumn::OriginalValue.as_str(),
                |c| c,
            )
            .column(
                TagFieldColumn::NewValue,
                TagFieldColumn::NewValue.as_str(),
                |c| c,
            )
            .on_submit(|s: &mut Cursive, _row: usize, index: usize| {
                let selected_field = s
                    .call_on_name(
                        METADATA_TABLE,
                        |table_view: &mut TableView<TagFieldView, TagFieldColumn>| {
                            let item = table_view.borrow_item(index).unwrap();
                            item.clone()
                        },
                    )
                    .unwrap();
                UiEventService::instance().send(UiEvent::SubmitMetadataRow(selected_field));
            })
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
            .child(Button::new("Add Folder", |_| {
                UiEventService::instance().send(UiEvent::OpenAddFolder);
            }))
            .child(Button::new("Add Files", |_| {
                UiEventService::instance().send(UiEvent::OpenAddFile);
            }))
            .child(Button::new("Cluster", Cursive::noop))
            .child(Button::new("Lookup", Cursive::noop))
            .child(Button::new("Scan", Cursive::noop))
            .child(Button::new("Save", Cursive::noop))
            .child(Button::new("Info", Cursive::noop))
            .child(Button::new("Remove", Cursive::noop))
            .child(Button::new("Lookup CD", Cursive::noop));
        let top_level_layout = LinearLayout::vertical()
            .child(file_panes)
            .child(hideable_bottom_panel)
            .child(bottom_buttons);
        root.add_fullscreen_layer(top_level_layout);
    }

    pub fn root(&self) -> &Mutex<CursiveRunnable> {
        &self.root
    }

    pub fn run(self) {
        self.root.lock().unwrap().run();
    }
}

impl Default for CursiveWrapper {
    fn default() -> Self {
        Self::new()
    }
}
