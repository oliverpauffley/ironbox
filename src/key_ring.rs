use std::{collections::HashMap, fs, io, path::Path};

use base64::{engine::general_purpose, Engine};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

const KEY_LENGTH: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(into = "KeyRingFile")]
#[serde(from = "KeyRingFile")]
pub struct KeyRing {
    key_entries: HashMap<KeyID, KeyEntry>,
    file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRingFile {
    #[serde(rename = "keyentries")]
    key_entries: Vec<KeyEntry>,
    file_path: String,
}

impl From<KeyRingFile> for KeyRing {
    fn from(file: KeyRingFile) -> Self {
        Self {
            key_entries: HashMap::from_iter(
                file.key_entries
                    .iter()
                    .map(|key_entry| (key_entry.key_id.clone(), key_entry.clone())),
            ),
            file_path: file.file_path,
        }
    }
}

impl From<KeyRing> for KeyRingFile {
    fn from(ring: KeyRing) -> Self {
        Self {
            key_entries: Vec::from_iter(ring.key_entries.iter().map(|key| key.1.clone())),
            file_path: ring.file_path,
        }
    }
}

impl KeyRing {
    pub fn new(key_entries: HashMap<KeyID, KeyEntry>, file_path: String) -> Self {
        Self {
            key_entries,
            file_path,
        }
    }

    pub fn load<P: AsRef<Path>>(file_path: P) -> Result<Self, io::Error> {
        let f = fs::File::open(file_path)?;

        let d = serde_yaml::from_reader(f).unwrap();

        Ok(d)
    }

    pub fn save<P: AsRef<Path>>(&self, file_path: P) -> Result<(), io::Error> {
        let f = fs::File::create(file_path)?;
        // TODO error
        serde_yaml::to_writer(f, self).unwrap();

        Ok(())
    }

    pub fn add_key(&mut self, description: String, key_id: String, key: String) {
        self.key_entries.insert(
            KeyID::new(key_id.clone()),
            KeyEntry::new(description, key, key_id),
        );
    }

    fn key(&self, key_id: String) -> Result<String, String> {
        let b64 = general_purpose::STANDARD;
        let key = self
            .key_entries
            .get(&KeyID::new(key_id.clone()))
            .ok_or(format!("could not find {} in key ring", key_id))?;
        let bytes = b64
            .decode(&key.key.0)
            .map_err(|err| format!("could not decode key: {}", err))?;
        // TODO fix this error
        let value = String::from_utf8(bytes).unwrap();
        Ok(value)
    }

    pub fn gen_key(&mut self, description: String) -> Result<(), String> {
        let mut rng = thread_rng();

        let key: String = (0..KEY_LENGTH).map(|_| rng.gen::<char>()).collect();

        let key_id = sha256::digest(key.clone());

        self.add_key(description, key_id, key);

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEntry {
    description: String,
    key: Key,
    #[serde(rename = "key-id")]
    key_id: KeyID,
}

impl KeyEntry {
    fn new(description: String, key: String, key_id: String) -> Self {
        Self {
            description,
            key: Key::new(key),
            key_id: KeyID::new(key_id),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct KeyID(String);

impl KeyID {
    fn new(key_id: String) -> Self {
        let b64 = general_purpose::STANDARD;
        Self(b64.encode(key_id))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Key(String);

impl Key {
    fn new(key: String) -> Self {
        let b64 = general_purpose::STANDARD;
        Self(b64.encode(key))
    }
}
