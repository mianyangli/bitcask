//! A simple in-memory key-value store.
#![deny(missing_docs)]
use std::collections::HashMap;
use failure::{format_err, Error};
use std::path::PathBuf;
use std::fs::{File, OpenOptions};
 use std::io::{BufRead, BufReader, Read, Seek, SeekFrom,
  Write};
use serde::{Deserialize, Serialize};

const LOG_FILE: &str = "kvs.log";
const COMPACTION_THRESHOLD: u64 = 1024 * 1024;
/// Library-wide result type.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Serialize, Deserialize)]
enum Command {
    Set { key: String, value: String },
    Rm { key: String },
}
struct CommandPos {
    pos: u64,
    len: u64
}


/// An in-memory key-value store
pub struct KvStore {
    data: HashMap<String, CommandPos>,
    path: PathBuf,
    uncompacted: u64,
}



impl KvStore {
    fn compact(&mut self) -> Result<()> {
        let compact_path = self.path.with_extension("compact");
        let mut compact_file = OpenOptions::new()
          .create(true)
          .write(true)
          .truncate(true)
          .open(&compact_path)?;

        let mut new_data: HashMap<String, CommandPos> = HashMap::new();
        let mut new_pos = 0u64;

        for key in self.data.keys().cloned().collect::<Vec<_>>() {
            if let Some(value) = self.get(key.clone())? {
                let cmd = Command::Set {
                  key: key.clone(),
                  value,
                };
                
                let serialized = serde_json::to_string(&cmd)?;
                writeln!(compact_file, "{}", serialized)?;

                let len = serialized.len() as u64 + 1;
                new_data.insert(key, CommandPos { pos: new_pos, len });
                new_pos += len;

            }
        }

        std::fs::rename(compact_path, &self.path)?;
        self.data = new_data;
        self.uncompacted = 0;


        Ok(())
    }
}


impl Default for KvStore {
    fn default() -> Self {
        Self::new()
    }
}

impl KvStore {
    /// ```
    /// use kvs::KvStore;
    ///
    /// let store = KvStore::new();
    /// assert_eq!(store.get("missing".to_string())?, None);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new() -> KvStore {
        KvStore {
            data: HashMap::new(),
            path: PathBuf::new(),
            uncompacted: 0,
        }
    }
}


impl KvStore {
    /// ```
    /// use kvs::KvStore;
    /// use tempfile::TempDir;
    ///
    /// let temp_dir = TempDir::new()?;
    /// let store = KvStore::open(temp_dir.path())?;
    /// assert_eq!(store.get("missing".to_string())?, None);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn open(_path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = _path.into();
        let log_path = path.join(LOG_FILE);
        let mut _data = HashMap::new();
        let mut uncompacted= 0;

        if log_path.exists() {
            let file = File::open(&log_path)?;
            let mut reader = BufReader::new(file);
            let mut pos = 0u64;
            loop {
                let mut line = String::new();
                let len = reader.read_line(&mut line)?;
                if len == 0 {
                    break;
                }
                
                let cmd: Command = serde_json::from_str(&line)?;
                let new_pos = CommandPos {
                    pos,
                    len: len as u64,
                };

                match cmd {
                    Command::Set { key, ..} => {
                        if let Some(old_cmd) = _data.insert(key, new_pos) {
                            uncompacted += old_cmd.len;
                        }
                    }
                    Command::Rm { key } => {
                        if let Some(old_cmd) = _data.remove(&key) {
                            uncompacted += old_cmd.len;
                        }
                        uncompacted += len as u64;
                    }
                }
                pos += len as u64;

            }
        }



        Ok(KvStore {
        data: _data,
        path: log_path,
        uncompacted,
        })

    }
}




impl KvStore {
    /// ```
    /// use kvs::KvStore;
    /// use tempfile::TempDir;
    ///
    /// let temp_dir = TempDir::new()?;
    /// let mut store = KvStore::open(temp_dir.path())?;
    /// store.set("key".to_string(), "value".to_string())?;
    /// assert_eq!(store.get("missing".to_string())?, None);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd = Command::Set { key: key.clone(), value: value.clone(), };
        let serialized = serde_json::to_string(&cmd)?;
        let mut file = OpenOptions::new().create(true).append(true).open(&self.path)?;
        let pos = file.seek(SeekFrom::End(0))?;
        writeln!(file, "{}", serialized)?;

        let new_pos = CommandPos {
            pos,
            len: serialized.len() as u64 + 1,
        };

        if let Some(old_pos) = self.data.insert(key, new_pos) {
            self.uncompacted += old_pos.len;
        }
        if self.uncompacted > COMPACTION_THRESHOLD {
            self.compact()?;
        }
        Ok(())
    }
}

impl KvStore {
    /// ```
    /// use kvs::KvStore;
    /// use tempfile::TempDir;
    ///
    /// let temp_dir = TempDir::new()?;
    /// let mut store = KvStore::open(temp_dir.path())?;
    /// store.set("key".to_string(), "value".to_string())?;
    ///
    /// assert_eq!(store.get("key".to_string())?, Some("value".to_string()));
    /// assert_eq!(store.get("missing".to_string())?, None);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```

    pub fn get(&self, key: String) -> Result<Option<String>> {
        let cmd_pos = match self.data.get(&key) {
            Some(pos) => pos, None => return Ok(None),
        };
        let mut file = File::open(&self.path)?;
        file.seek(SeekFrom::Start(cmd_pos.pos))?;
        let mut buf = vec![0; cmd_pos.len as usize];
        file.read_exact(&mut buf)?;
        if buf.last() == Some(&b'\n') {
            buf.pop();
        }
        let cmd: Command = serde_json::from_slice(&buf)?;
        match cmd {
            Command::Set { value, ..} => Ok(Some(value)),
            Command::Rm { .. } => Ok(None),
        }
    }
}

impl KvStore {
        /// ```
    /// use kvs::KvStore;
    /// use tempfile::TempDir;
    ///
    /// let temp_dir = TempDir::new()?;
    /// let mut store = KvStore::open(temp_dir.path())?;
    /// store.set("key".to_string(), "value".to_string())?;
    /// store.remove("key".to_string())?;
    ///
    /// assert_eq!(store.get("key".to_string())?, None);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn remove(&mut self, key: String) -> Result<()> {
        let old_pos = match self.data.remove(&key) {
            Some(old_pos) => old_pos,
            None => return Err(format_err!("Key not found")),
        };

        let cmd = Command::Rm {key: key.clone()};
        let serialized = serde_json::to_string(&cmd)?;
        let mut file = OpenOptions::new().create(true).append(true).open(&self.path)?;
        writeln!(file, "{}", serialized)?;

        self.uncompacted += old_pos.len;
        self.uncompacted += serialized.len() as u64 + 1;

        if self.uncompacted > COMPACTION_THRESHOLD {
            self.compact()?;
        }

        Ok(())
    }
}
