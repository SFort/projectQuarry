#![allow(dead_code)]
use std::sync::mpsc;
pub mod file;
pub fn update(path: &str, url: &str, log: &mut Loghelper) {
    use git2::Repository;
    log.title("Processing git download");
    let repo = match Repository::open(path) {
        Ok(repo) => {
            log.desc("Opened Repository");
            repo
        }
        Err(_) => {
            log.desc("Creating new Repository");
            Repository::clone(url, path).unwrap()
        }
    };
    let mut remote = match repo.find_remote("origin") {
        Ok(r) => {
            log.desc("Using existing remote");
            r
        }
        Err(_) => {
            log.desc("Registered new remote");
            repo.remote("origin", url).unwrap()
        }
    };
    match remote.download(&[""], None) {
        Ok(_) => log.desc("Started Download"),
        Err(e) => log.desc(format!("Download Failed\n{}", e)),
    }
    match remote.fetch(&["master"], None, None) {
        Ok(_) => log.desc("Fetching.."),
        Err(e) => log.desc(format!("Failed to fetch files\n{}", e)),
    }
    let oid = repo.refname_to_id("refs/remotes/origin/master").unwrap();
    let object = repo.find_object(oid, None).unwrap();
    match repo.reset(&object, git2::ResetType::Hard, None) {
        Ok(_) => log.desc("Set repo to Latest"),
        Err(e) => log.desc(format!("could not set to Latest\n{}", e)),
    }
}
impl Loghelper {
    pub fn title<S>(&mut self, title: S)
    where
        S: Into<String>,
    {
        self.title = title.into();
        self.log
            .send(Some((self.title.clone(), "".into())))
            .unwrap();
    }
    pub fn desc<S>(&mut self, desc: S)
    where
        S: Into<String>,
    {
        self.desc = desc.into();
        self.log
            .send(Some((self.title.clone(), self.desc.clone())))
            .unwrap();
    }
    pub fn new(log: mpsc::Sender<Option<(String, String)>>) -> Loghelper {
        Loghelper {
            log,
            title: "".into(),
            desc: "".into(),
        }
    }
    pub fn none(&self) {
        self.log.send(None).unwrap();
    }
}
pub struct Loghelper {
    log: mpsc::Sender<Option<(String, String)>>,
    title: String,
    desc: String,
}
