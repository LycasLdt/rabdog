use std::{
    borrow::Cow,
    io::{Cursor, Read as _},
};

use anyhow::{anyhow, Result};
use quick_xml::{events::Event, Reader};

pub fn get_next_data(text: &str) -> Result<String> {
    let mut reader = Reader::from_str(text);
    reader.trim_text(true);

    let mut is_in_data = false;

    loop {
        match reader.read_event()? {
            Event::Start(start) => {
                is_in_data = false;

                let tag = start.name();
                let id = start.try_get_attribute("id")?.map(|attr| attr.value);
                if tag.as_ref() == b"script" && id == Some(Cow::Borrowed(b"__NEXT_DATA__")) {
                    is_in_data = true;
                }
            }
            Event::Text(txt) => {
                if is_in_data {
                    return Ok(txt.unescape()?.into_owned());
                }
            }
            Event::Eof => break,
            _ => (),
        }
    }

    Err(anyhow!("__NEXT_DATA__ is missing"))
}

pub fn get_sb3_project<I: AsRef<[u8]>>(input: I) -> Result<Vec<u8>> {
    let cursor = Cursor::new(input.as_ref());
    let mut archive = zip::ZipArchive::new(cursor)?;
    let mut buf = Vec::new();

    archive.by_name("project.json")?.read_to_end(&mut buf)?;
    Ok(buf)
}
