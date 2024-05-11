use std::io::{Cursor, Read, Seek, Write};

use anyhow::{Ok, Result};
use serde::Deserialize;
use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};

#[derive(Deserialize)]
pub struct Sb3Project {
    pub targets: Vec<Sb3Target>,
}
#[derive(Deserialize)]
pub struct Sb3Target {
    pub costumes: Vec<Sb3Asset>,
    pub sounds: Vec<Sb3Asset>,
}
#[derive(Deserialize)]
pub struct Sb3Asset {
    #[serde(skip)]
    pub kind: Sb3AssetKind,
    pub md5ext: String,
}

#[derive(Clone, Copy)]
pub enum Sb3AssetKind {
    Costume,
    Sound,
}
impl Default for Sb3AssetKind {
    fn default() -> Self {
        Self::Costume
    }
}

pub struct Sb3Reader(pub Vec<u8>);
impl Sb3Reader {
    pub fn from_zip<B: AsRef<[u8]>>(buf: B) -> Result<Self> {
        let cursor = Cursor::new(buf.as_ref());
        let mut archive = ZipArchive::new(cursor)?;
        let mut buf = Vec::new();

        archive.by_name("project.json")?.read_to_end(&mut buf)?;

        Ok(Sb3Reader(buf))
    }
    pub fn parse<J: AsRef<[u8]>>(json: J) -> Self {
        Sb3Reader(json.as_ref().into())
    }
    pub fn to_project(&self) -> Result<Sb3Project> {
        Ok(serde_json::from_slice::<Sb3Project>(&self.0)?)
    }

    pub fn assets(&self) -> Result<Vec<Sb3Asset>> {
        let targets = self.to_project()?.targets;
        let assets = targets.into_iter().flat_map(|target| {
            fn with_kind(assets: Vec<Sb3Asset>, kind: Sb3AssetKind) -> Vec<Sb3Asset> {
                assets
                    .into_iter()
                    .map(|asset| Sb3Asset { kind, ..asset })
                    .collect()
            }
            let mut assets = with_kind(target.costumes, Sb3AssetKind::Costume);
            assets.extend(with_kind(target.sounds, Sb3AssetKind::Sound));
            assets
        });

        Ok(assets.collect())
    }
}

pub struct Sb3Writer<W: Write + Seek> {
    inner: ZipWriter<W>,
    assets: Vec<String>
}
impl<W: Write + Seek> Sb3Writer<W> {
    pub fn new(writer: W) -> Self {
        let inner = ZipWriter::new(writer);
        Sb3Writer { inner, assets: Vec::new() }
    }

    pub fn set_project_json<C: AsRef<[u8]>>(&mut self, json: C) -> Result<&mut Self> {
        self.add_asset("project.json", json.as_ref())?;
        Ok(self)
    }
    pub fn add_asset(&mut self, name: &str, buf: &[u8]) -> Result<&mut Self> {
        let inner = &mut self.inner;
        if !self.assets.contains(&name.to_owned()) {
            inner.start_file(name, SimpleFileOptions::default())?;
            inner.write_all(buf)?;

            self.assets.push(name.to_owned())
        }

        Ok(self)
    }
    pub fn finish(&mut self) -> Result<W> {
        Ok(self.inner.finish()?)
    }
}
