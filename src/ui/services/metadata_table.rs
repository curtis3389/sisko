use super::CbSinkService;
use crate::{
    domain::{
        models::{AudioFile, Tag, TagField, TagType},
        repos::TagRepository,
    },
    infrastructure::{merge, spawn, MergeAction, TableViewExtensions},
    ui::models::{TagFieldColumn, TagFieldView, METADATA_TABLE, NEW_FIELD_VALUE},
};
use anyhow::{anyhow, Result};
use cursive::{
    reexports::enumset::enum_set,
    theme::{ColorStyle, Effect, Style},
    traits::{Nameable, Resizable},
    views::{Dialog, EditView, LinearLayout, TextView},
    Cursive,
};
use cursive_table_view::TableView;
use log::error;

pub struct MetadataTable {}

impl MetadataTable {
    pub fn new() -> Self {
        Self {}
    }

    pub fn open_tag_field_dialog(
        &self,
        audio_file: AudioFile,
        tag_type: TagType,
        field: TagField,
    ) -> Result<()> {
        CbSinkService::instance()?
            .send(Box::new(move |s: &mut Cursive| {
                if let Err(e) = tag_field_dialog(s, audio_file, tag_type, field) {
                    error!("Error opening tag field dialog: {e}!");
                }
            }))
            .map_err(|_| anyhow!("Error sending open tag field dialog callback to CbSink!"))
    }

    pub fn set_metadata_table(&self, tags: &[Tag]) -> Result<()> {
        let items: Vec<TagFieldView> = tags
            .iter()
            .flat_map(|tag| tag.fields.iter().map(TagFieldView::new))
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

    pub fn update_metadata_table(&self, tags: &[Tag]) -> Result<()> {
        let items: Vec<TagFieldView> = tags
            .iter()
            .flat_map(|tag| tag.fields.iter().map(TagFieldView::new))
            .collect();
        CbSinkService::instance()?
            .send(Box::new(move |s: &mut Cursive| {
                s.call_on_name(
                    METADATA_TABLE,
                    |table: &mut TableView<TagFieldView, TagFieldColumn>| {
                        let (to_add, to_remove, to_update) = {
                            let actions = merge(table.borrow_items(), &items);
                            let to_add: Vec<TagFieldView> = actions
                                .iter()
                                .filter_map(|action| match &action {
                                    MergeAction::Add(new) => Some((*new).clone()),
                                    _ => None,
                                })
                                .collect();
                            let to_remove: Vec<TagFieldView> = actions
                                .iter()
                                .filter_map(|action| match &action {
                                    MergeAction::Remove(old) => Some((*old).clone()),
                                    _ => None,
                                })
                                .collect();
                            let to_update: Vec<TagFieldView> = actions
                                .iter()
                                .filter_map(|action| match &action {
                                    MergeAction::Update(new) => Some((*new).clone()),
                                    _ => None,
                                })
                                .collect();

                            (to_add, to_remove, to_update)
                        };

                        to_add.into_iter().for_each(|new| table.insert_item(new));
                        to_remove.into_iter().for_each(|old| {
                            if let Some(index) = table.index_of(|item| item.id == old.id) {
                                table.remove_item(index);
                            }
                        });
                        to_update.into_iter().for_each(|new| {
                            if let Some(index) = table.index_of(|item| item.id == new.id) {
                                if let Some(item) = table.borrow_item_mut(index) {
                                    *item = new;
                                }
                            }
                        });
                    },
                );
            }))
            .map_err(|_| anyhow!("Error sending set metadata table callback to CbSink!"))
    }
}

impl Default for MetadataTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Opens a tag field edit dialog for the given tag field view.
///
/// # Arguments
///
/// * `s` - The Cursive to open the dialog with.
/// * `view` - The tag field view to open the dialog for.
fn tag_field_dialog(
    s: &mut Cursive,
    audio_file: AudioFile,
    tag_type: TagType,
    field: TagField,
) -> Result<()> {
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
            let audio_file = audio_file.clone();
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
                spawn(async move {
                    let repo = TagRepository::instance();
                    let mut tag = repo.get(&audio_file, &tag_type).await?;
                    tag.update_field(field);
                    repo.save(tag).await
                });
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
