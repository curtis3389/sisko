use sisko_lib::id3v2_frame::ID3v2Frame;
use sisko_lib::id3v2_frame_fields::ID3v2FrameFields;
use sisko_lib::id3v2_tag::ID3v2Tag;
use std::fs::File;
use std::io::Write;

/// This is the entrypoint of the program.
fn main() {
    let tag = ID3v2Tag::read_from_path("./ref/01_Ghosts_I_64kb.mp3").unwrap();
    println!("{:#?}", tag);
    let mut apic = tag
        .frames
        .iter()
        .filter(|f| f.header.frame_id == "APIC")
        .collect::<Vec<&ID3v2Frame>>();

    if apic.len() != 0 {
        let apic = apic.remove(0);
        match &apic.fields {
            ID3v2FrameFields::AttachedPictureFields {
                picture_data,
                encoding: _,
                mime_type: _,
                picture_type: _,
                description: _,
            } => {
                let mut file = File::create("other.jpg").unwrap();
                file.write_all(&picture_data).unwrap();
            }
            _ => panic!(),
        }
    }
}
