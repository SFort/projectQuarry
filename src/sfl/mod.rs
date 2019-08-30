#![allow(dead_code)]
use std::sync::mpsc;
pub fn update(path: &str, url: &str, logger: &mpsc::Sender<Option<String>>) {
    use git2::Repository;
    let repo = match Repository::open(path) {
        Ok(repo) => {
            logger.send(Some("Orened Repository".into())).unwrap();
            repo
        }
        Err(_) => Repository::clone(url, path).unwrap(),
    };
    let mut remote = match repo.find_remote("origin") {
        Ok(r) => {
            logger
                .send(Some("Opened existing Repository".into()))
                .unwrap();
            r
        }
        Err(_) => {
            logger
                .send(Some("Registered new Repository".into()))
                .unwrap();
            repo.remote("origin", url).unwrap()
        }
    };
    match remote.download(&[""], None) {
        Ok(_) => logger.send(Some("Started Download".into())).unwrap(),
        Err(e) => logger
            .send(Some(format!("Download Failed\n{}", e)))
            .unwrap(),
    }
    match remote.fetch(&["master"], None, None) {
        Ok(_) => logger.send(Some("Fetching..".into())).unwrap(),
        Err(e) => logger
            .send(Some(format!("Failed to fetch files\n{}", e)))
            .unwrap(),
    }
    let oid = repo.refname_to_id("refs/remotes/origin/master").unwrap();
    let object = repo.find_object(oid, None).unwrap();
    match repo.reset(&object, git2::ResetType::Hard, None) {
        Ok(_) => logger.send(Some("Set repo to Latest".into())).unwrap(),
        Err(e) => logger
            .send(Some(format!("could not set to Latest\n{}", e)))
            .unwrap(),
    }
}
pub fn robcop(from: &str, to: &str, dir: &str, mir: bool) {
    use std::process::Command;
    Command::new("cmd")
        .args(&[
            "/C",
            &format!(
                "robocopy {1}/{2} {0}/{2} /{3}",
                to,
                from,
                dir,
                if mir { "mir" } else { "s" }
            ),
        ])
        .output()
        .expect("failed to robocopy");
}
