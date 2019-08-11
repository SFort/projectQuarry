#![allow(dead_code)]
pub fn update(path: &str, url: &str) {
    use git2::Repository;
    let repo = match Repository::open(path) {
        Ok(repo) => repo,
        Err(_) => Repository::clone(url, path).unwrap(),
    };
    let mut remote = match repo.find_remote("origin") {
        Ok(r) => r,
        Err(_) => repo.remote("origin", url).unwrap(),
    };
    remote.download(&[""], None).unwrap();
    remote.fetch(&["master"], None, None).unwrap();
    let oid = repo.refname_to_id("refs/remotes/origin/master").unwrap();
    let object = repo.find_object(oid, None).unwrap();
    repo.reset(&object, git2::ResetType::Hard, None).unwrap();
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
