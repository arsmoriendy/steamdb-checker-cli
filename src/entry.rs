use crate::prelude::*;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Entry {
    pub name: String,
    #[serde(rename = "SHA1 Hash")]
    pub sha1_hash: String,
}

pub type Entries = HashMap<PathBuf, Option<String>>;

impl Entry {
    pub async fn list(csv: &Path) -> Result<Entries> {
        let mut entries: Entries = HashMap::new();
        for entry in csv::Reader::from_path(csv)?.deserialize::<Entry>() {
            let Entry { name, sha1_hash } = entry?;
            entries.insert(
                name.into(),
                match sha1_hash.len() {
                    0 => None,
                    _ => Some(sha1_hash),
                },
            );
        }
        Ok(entries)
    }
}
