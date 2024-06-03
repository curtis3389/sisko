use crate::domain::{ISiskoService, ITrackService};
use crate::ui::*;
use cursive::event::{Event, Key};
use cursive::traits::*;
use cursive::views::{Button, HideableView, LinearLayout, NamedView, Panel, ResizedView};
use cursive::{menu, Cursive, CursiveRunnable};
use cursive_table_view::TableView;
use syrette::DIContainer;

/// Represents the host for the UI.
pub struct UiHost {
    /// The dependency injection container for the app.
    container: DIContainer,
}

impl UiHost {
    /// Returns a new UI host.
    ///
    /// # Arguments
    ///
    /// * `root` - The cursive root to create the UI host with.
    pub fn new(container: DIContainer) -> Self {
        let cursive = container
            .get::<dyn ICursive>()
            .unwrap()
            .singleton()
            .unwrap();
        let mut root = cursive.root().lock().unwrap();
        Self::add_hotkeys(&mut root);
        Self::setup_menubar(&mut root);
        Self::add_widgets(&mut root);
        Self { container }
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
                    Panel<ResizedView<NamedView<TableView<TagFieldView, TagFieldColumn>>>>,
                >| {
                    hideable.set_visible(!hideable.is_visible());
                },
            );
        });
        root.add_global_callback(Event::CtrlChar('l'), |s| {
            s.call_on_name(
                HIDEABLE_LEFT_PANEL,
                |hideable: &mut HideableView<
                    Panel<ResizedView<NamedView<TableView<TrackView, TrackColumn>>>>,
                >| {
                    hideable.set_visible(!hideable.is_visible());
                },
            );
        });
        root.add_global_callback(Event::CtrlChar('r'), |s| {
            s.call_on_name(
                HIDEABLE_RIGHT_PANEL,
                |hideable: &mut HideableView<
                    Panel<ResizedView<NamedView<TableView<TrackView, TrackColumn>>>>,
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
                let container = s.user_data().unwrap() as &mut DIContainer;
                let track_service = container
                    .get::<dyn ITrackService>()
                    .unwrap()
                    .singleton()
                    .unwrap();
                let track = track_service.get(&selected_track.file);
                let sisko_service = container
                    .get::<dyn ISiskoService>()
                    .unwrap()
                    .transient()
                    .unwrap();
                sisko_service.select_track(&track);
            })
            .with_name(CLUSTER_FILE_TABLE);

        let album_file_table = TableView::<TrackView, TrackColumn>::new()
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
                let ui = container.get::<dyn Ui>().unwrap().singleton().unwrap();
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

/// Does nothing.
fn do_nothing(_: &mut Cursive) {}
