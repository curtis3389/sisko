- file: a file or directory we've looked at (e.g. Rubber\ Soul/)
- track: a file with audio data and tags we can read (e.g. track1.mp3)
- tag: a block of metadata with fields like artist and title (e.g. an ID3v2 tag)
- tag field: a single key-value in a tag (e.g. Artist: Joy Division)
- tag field change: a change to a tag field (e.g. add album artist tag field)
- tag change: a change to a tag as a whole (e.g. change from v2.3.0 to v2.4.0)
- track pairing: a pairing of a track to a the metadata of a particular recording-release combo
- track metadata: metadata about a track from MusicBrainz (i.e. recording metadata)
- album metadata: metadata about an album from MusicBrainz (i.e. release metadata)
- left pane: contains tracks that haven't been paired
- right pane: contains releases with track pairings
- bottom pane: contains tag field changes

```rust
Track {
    file_id: Uuid(Path),
    tags: Tag[],
}

Tag {
    file_id: Uuid(Path),
    tag_type: TagType,
    fields: TagField[],
}

TagField {
    file_id: Uuid(Path),
    tag_type: TagType,
    field_id: Uuid(file_id: Uuid, tag_type: TagType, field_name: String),
    field_value: TagFieldValue,
}

TagFieldValue {
    TextValue(),
    UserTextValue(),
    ...
}

TrackPairing {
    file_id: Uuid(Path),
    metadata: Option<TrackMetadata>,
}

TagFieldChange {
    tag_field: TagField,
    new_value: Option<TagValue>,
}
```

events:

* user loaded a file
* user loaded a directory
* user changed a field's value
* user paired a track to metadata
* user saved a file

scan file:
1. fingerprint file
1. get recording id for fingerprint from acoustid
1. load recording and release metadata into memory
1. pair file with a particular release
