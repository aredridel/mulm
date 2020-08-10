use crate::err::ListError;
use crate::send::send;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use fs2::FileExt;
use maildir::Maildir;
use mailparse::{addrparse, MailAddr, SingleInfo};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::error::Error;
use std::fmt;
use std::fs::{hard_link, metadata, rename, DirBuilder, File, OpenOptions};
use std::io::prelude::*;
use std::io::{BufRead, BufReader, ErrorKind};
use std::path::Path;
use toml;

#[derive(Debug, Eq, PartialEq)]
pub enum MailingListAction<'a> {
    Subscribe(String),
    Unsubscribe(String),
    Message(&'a [u8]),
    Reject,
}

#[derive(Deserialize, Serialize, Debug)]
struct Root {
    config: Config,
}

pub struct List {
    maildir: Maildir,
    config: Config,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    name: String,
    slug: String,
    open_posting: Option<bool>,
    tag_subject: Option<bool>,
}

impl std::fmt::Debug for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "List {{ path: {:?}, config: {:?} }}",
            self.maildir.path(),
            self.config
        )
    }
}

impl List {
    pub fn load<P: AsRef<Path>>(dirname: P) -> Result<List, Box<dyn Error>> {
        let dir = Path::new(dirname.as_ref());
        let config_path = dir.join("config.toml");
        let mut file = File::open(config_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let temp: Root = toml::from_str(&contents)?;
        let list = List {
            maildir: Maildir::from(dirname.as_ref().to_str().unwrap()),
            config: temp.config,
        };
        Ok(list)
    }

    pub fn is_subscribed(&self, address: &str) -> Result<bool, Box<dyn Error>> {
        let check = parse_addr(address)?;
        let sub_path = Path::new(&self.maildir.path()).join("subscriptions");
        let subscriptions = match OpenOptions::new().read(true).open(sub_path) {
            Ok(subs) => subs,
            Err(err) => {
                return match err.kind() {
                    ErrorKind::NotFound => Ok(false),
                    _ => Err(Box::new(err)),
                }
            }
        };
        subscriptions.lock_shared()?;

        for line in BufReader::new(&subscriptions).lines() {
            let subs = addrparse(line?.as_str())?;
            for sub in subs.iter() {
                if let MailAddr::Single(x) = sub {
                    if x.addr == check.addr {
                        return Ok(true);
                    }
                }
            }
        }

        subscriptions.unlock()?;
        Ok(false)
    }

    pub fn subscribe(&self, address: String) -> Result<(), Box<dyn Error>> {
        if !self.is_subscribed(&address)? {
            let mut subscriptions = OpenOptions::new()
                .append(true)
                .create(true)
                .open(Path::new(&self.maildir.path()).join("subscriptions"))?;
            subscriptions.lock_shared()?;
            write!(&mut subscriptions, "{}\n", address)?;
            subscriptions.unlock()?;
        }
        Ok(())
    }

    pub fn unsubscribe(&self, address: String) -> Result<(), Box<dyn Error>> {
        let old_name = Path::new(&self.maildir.path()).join("subscriptions");
        let new_name = Path::new(&self.maildir.path()).join("subscriptions.new");
        let old = File::open(&old_name)?;
        let mut newsubs = File::create(&new_name)?;
        old.lock_exclusive()?;

        for maybe_line in BufReader::new(&old).lines() {
            let line = maybe_line?;
            let trimmed = line.trim();
            if trimmed != address {
                write!(&mut newsubs, "{}\n", trimmed)?;
            }
        }
        rename(new_name, old_name)?;
        old.unlock()?;
        Ok(())
    }

    pub fn send(&self, message_buf: &[u8]) -> Result<String, Box<dyn Error>> {
        self.maildir.create_dirs()?;

        let message = mailparse::parse_mail(message_buf)?;

        let mut buf: Vec<u8> = Vec::new();

        for header in &message.headers {
            buf.write_all(header.get_key_raw())?;
            buf.write_all(b": ")?;
            if self.config.tag_subject.unwrap_or(false)
                && header.get_key().eq_ignore_ascii_case("subject")
            {
                write!(buf, "[{}] ", self.config.slug)?;
            }
            buf.write_all(header.get_value_raw())?;
            buf.write_all(b"\r\n")?;
        }
        buf.write_all(b"\r\n")?;
        buf.write_all(&message.get_body_raw()?)?;

        let id = self.maildir.store_new(&buf)?;

        DirBuilder::new()
            .recursive(true)
            .create(self.maildir.path().join("queue"))?;
        if let Some(entry) = self.maildir.find(&id) {
            hard_link(
                entry.path(),
                self.maildir.path().join(format!("queue/{}.eml", id)),
            )?;
        } else {
            return Err(Box::new(ListError {
                message: "Newly added message vanished!".to_string(),
            }));
        }

        let subscriptions_filename = Path::new(&self.maildir.path()).join("subscriptions");
        let message_destinations_filename = Path::new(&self.maildir.path())
            .join("queue")
            .join(format!("{}.dest", id));
        let subscriptions = match OpenOptions::new().read(true).open(&subscriptions_filename) {
            Ok(subs) => subs,
            Err(err) => match err.kind() {
                ErrorKind::NotFound => OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create_new(true)
                    .open(&subscriptions_filename)?,
                _ => return Err(Box::new(err)),
            },
        };
        subscriptions.lock_shared()?;
        hard_link(&subscriptions_filename, &message_destinations_filename)?;

        let stat = metadata(&subscriptions_filename)?;
        let stat_filename = Path::new(&self.maildir.path())
            .join("queue")
            .join(format!("{}.pos", id));
        let mut pos = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(stat_filename)?;
        pos.write_u64::<BigEndian>(0)?;
        pos.write_u64::<BigEndian>(stat.len())?;

        subscriptions.unlock()?;

        // TODO Spawn send process and pass it the list directory and {id}
        // println!("Would have started send process here, running inline for now");
        self.dequeue_and_send_message(&id)?;

        Ok(id)
    }

    fn dequeue_and_send_message(&self, id: &str) -> Result<(), Box<dyn Error>> {
        let pos_file = self
            .maildir
            .path()
            .join("queue")
            .join(format!("{}.pos", id));
        let mut pos = OpenOptions::new().read(true).write(true).open(&pos_file)?;
        pos.lock_shared()?;

        // Confirm the file is still here: if another process handled it, it will vanish before
        // being unlocked, so we'll end up here.
        if !pos_file.exists() {
            return Ok(());
        }

        let mut curr_pos = pos.read_u64::<BigEndian>()?;
        let end = pos.read_u64::<BigEndian>()?;

        let dest_list_file = self
            .maildir
            .path()
            .join("queue")
            .join(format!("{}.dest", id));
        let mut dest_list = BufReader::new(File::open(&dest_list_file)?);
        dest_list.seek(std::io::SeekFrom::Start(curr_pos))?;

        let message_file = self
            .maildir
            .path()
            .join("queue")
            .join(format!("{}.eml", id));
        let mut message = vec![];
        File::open(&message_file)?.read_to_end(&mut message)?;

        while curr_pos < end {
            let mut buf = String::new();
            let incr: u64 = dest_list.read_line(&mut buf)?.try_into().unwrap();
            curr_pos += incr;

            send(None, &[buf.trim()], &message)?;

            pos.seek(std::io::SeekFrom::Start(curr_pos))?;
            pos.write_u64::<BigEndian>(curr_pos)?;
            pos.sync_data()?;
        }

        let delete1 = std::fs::remove_file(pos_file);
        let delete2 = std::fs::remove_file(dest_list_file);
        let delete3 = std::fs::remove_file(message_file);

        not_found_is_fine(delete1)?;
        not_found_is_fine(delete2)?;
        not_found_is_fine(delete3)?;

        pos.unlock()?;
        Ok(())
    }
}

fn parse_addr(addr: &str) -> Result<SingleInfo, Box<dyn Error>> {
    Ok(match addrparse(addr)?.remove(0) {
        MailAddr::Single(x) => x,
        MailAddr::Group(mut xs) => xs.addrs.remove(0),
    })
}

fn not_found_is_fine(r: Result<(), std::io::Error>) -> Result<(), Box<dyn Error>> {
    if let Err(e) = r {
        if e.kind() == ErrorKind::NotFound {
            return Ok(());
        } else {
            return Err(Box::new(e));
        }
    } else {
        return Ok(());
    }
}

#[cfg(test)]
mod test {
    use super::{Config, Root};
    use crate::list::List;
    use maildir::Maildir;
    use std::env::temp_dir;
    use std::error::Error;
    use std::fs::File;
    use std::io::Write;
    use toml;

    #[test]
    fn test_create_load() -> Result<(), Box<dyn Error>> {
        let dir = temp_dir();
        let cf = dir.join("config.toml");
        let mut config = File::create(cf)?;
        config.write_all(
            toml::to_string(&Root {
                config: Config {
                    name: "test".to_string(),
                    slug: "test".to_string(),
                    open_posting: None,
                    tag_subject: None,
                },
            })?
            .as_bytes(),
        )?;

        let list = List::load(dir)?;
        assert_eq!(list.config.name, "test");

        Ok(())
    }

    #[test]
    fn test_send() -> Result<(), Box<dyn Error>> {
        let dir = temp_dir();
        let cf = dir.join("config.toml");
        let mut config = File::create(cf)?;
        config.write_all(
            toml::to_string(&Root {
                config: Config {
                    name: "test".to_string(),
                    slug: "test".to_string(),
                    open_posting: None,
                    tag_subject: None,
                },
            })?
            .as_bytes(),
        )?;

        let list = List::load(&dir)?;

        let id = list.send(b"From: test@example.org\r\nSubject: a post\r\n\r\nTest\r\n")?;

        let maildir = Maildir::from(dir);

        maildir.find(&id);

        Ok(())
    }
}
