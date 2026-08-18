#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "core gate: a panic here is the report"
)]

mod support;

use std::path::{Path, PathBuf};

use support::archives::{Entry, gzipped, zipped};
use tolearn_core::archive::{ArchiveError, Limits, unpack};

const ROADMAP: &str = "id: demo\ntitle: Демо\nversion: 1\n";

fn workspace(name: &str) -> PathBuf {
    let room = std::env::temp_dir().join(format!("tolearn-archive-{name}-{}", std::process::id()));
    if room.exists() {
        std::fs::remove_dir_all(&room).unwrap();
    }
    std::fs::create_dir_all(&room).unwrap();
    room
}

fn bundle() -> Vec<Entry> {
    vec![
        Entry::file("roadmap.yaml", ROADMAP.as_bytes().to_vec()),
        Entry::file("topics/one.yaml", b"id: one\n".to_vec()),
    ]
}

fn taken(archive: &Path, room: &Path) -> PathBuf {
    unpack(archive, &room.join("out"), &Limits::DEFAULT).unwrap()
}

fn refusal(archive: &Path, room: &Path) -> ArchiveError {
    unpack(archive, &room.join("out"), &Limits::DEFAULT).unwrap_err()
}

fn within(archive: &Path, room: &Path, limits: &Limits) -> ArchiveError {
    unpack(archive, &room.join("out"), limits).unwrap_err()
}

#[test]
fn zip_отдаёт_корень_бандла() {
    let room = workspace("zip");
    let archive = zipped(&room, &bundle());

    let root = taken(&archive, &room);

    assert_eq!(
        std::fs::read_to_string(root.join("roadmap.yaml")).unwrap(),
        ROADMAP
    );
    assert!(root.join("topics/one.yaml").is_file());
}

#[test]
fn tar_gz_отдаёт_тот_же_корень_что_и_zip() {
    let room = workspace("targz");
    let archive = gzipped(&room, &bundle());

    let root = taken(&archive, &room);

    assert_eq!(
        std::fs::read_to_string(root.join("roadmap.yaml")).unwrap(),
        ROADMAP
    );
    assert!(root.join("topics/one.yaml").is_file());
}

#[test]
fn вложенный_корень_находится() {
    let room = workspace("nested");
    let deep: Vec<Entry> = bundle()
        .iter()
        .map(|entry| entry.under("выгрузка/bundle"))
        .collect();
    let archive = zipped(&room, &deep);

    let root = taken(&archive, &room);

    assert!(root.ends_with("bundle"), "{root:?}");
    assert!(root.join("roadmap.yaml").is_file());
}

#[test]
fn корня_бандла_в_архиве_может_не_быть() {
    let room = workspace("noroot");
    let archive = zipped(&room, &[Entry::file("readme.md", b"nothing".to_vec())]);

    assert!(matches!(refusal(&archive, &room), ArchiveError::NoBundle));
}

#[test]
fn два_корня_разбирать_не_за_кого() {
    let room = workspace("two");
    let mut entries: Vec<Entry> = bundle().iter().map(|entry| entry.under("a")).collect();
    entries.extend(bundle().iter().map(|entry| entry.under("b")));
    let archive = zipped(&room, &entries);

    assert!(matches!(
        refusal(&archive, &room),
        ArchiveError::ManyBundles
    ));
}

#[test]
fn путь_вверх_по_дереву_отвергается() {
    let room = workspace("updir");
    let mut entries = bundle();
    entries.push(Entry::file("../сбежал.yaml", b"boo".to_vec()));
    let archive = zipped(&room, &entries);

    assert!(matches!(
        refusal(&archive, &room),
        ArchiveError::Escaping { .. }
    ));
    assert!(!room.join("сбежал.yaml").exists(), "запись ушла за корень");
}

#[test]
fn абсолютный_путь_отвергается() {
    let room = workspace("absolute");
    let mut entries = bundle();
    entries.push(Entry::file("/tmp/сбежал.yaml", b"boo".to_vec()));
    let archive = gzipped(&room, &entries);

    assert!(matches!(
        refusal(&archive, &room),
        ArchiveError::Escaping { .. }
    ));
}

#[test]
fn симлинк_отвергается() {
    let room = workspace("symlink");
    let mut entries = bundle();
    entries.push(Entry::link("наружу", "/etc/passwd"));
    let archive = gzipped(&room, &entries);

    assert!(matches!(
        refusal(&archive, &room),
        ArchiveError::Link { .. }
    ));
}

#[test]
fn хардлинк_отвергается() {
    let room = workspace("hardlink");
    let mut entries = bundle();
    entries.push(Entry::hard("наружу", "/etc/passwd"));
    let archive = gzipped(&room, &entries);

    assert!(matches!(
        refusal(&archive, &room),
        ArchiveError::Link { .. }
    ));
}

#[test]
fn симлинк_в_zip_отвергается_тоже() {
    let room = workspace("zipsymlink");
    let mut entries = bundle();
    entries.push(Entry::link("наружу", "/etc/passwd"));
    let archive = zipped(&room, &entries);

    assert!(matches!(
        refusal(&archive, &room),
        ArchiveError::Link { .. }
    ));
}

#[test]
fn слишком_много_записей_не_распаковывается() {
    let room = workspace("many");
    let archive = zipped(&room, &bundle());

    let refused = within(
        &archive,
        &room,
        &Limits {
            entries: 1,
            ..Limits::DEFAULT
        },
    );

    assert!(matches!(refused, ArchiveError::TooMany { .. }));
}

#[test]
fn превышение_объёма_обрывает_распаковку() {
    let room = workspace("volume");
    let archive = zipped(&room, &bundle());

    let refused = within(
        &archive,
        &room,
        &Limits {
            bytes: 8,
            ..Limits::DEFAULT
        },
    );

    assert!(matches!(refused, ArchiveError::TooBig { .. }));
    assert!(!room.join("out").exists(), "недораспакованное осталось");
}

#[test]
fn слишком_плотное_сжатие_отвергается() {
    let room = workspace("ratio");
    let mut entries = bundle();
    entries.push(Entry::file("bomb.bin", vec![0; 1 << 20]));
    let archive = zipped(&room, &entries);

    let refused = within(
        &archive,
        &room,
        &Limits {
            ratio: 4,
            ..Limits::DEFAULT
        },
    );

    assert!(matches!(refused, ArchiveError::TooDense { .. }));
}

#[test]
fn обычный_бандл_в_потолки_укладывается() {
    let room = workspace("fits");
    let archive = zipped(&room, &bundle());

    assert!(unpack(&archive, &room.join("out"), &Limits::DEFAULT).is_ok());
}

#[test]
fn чужой_формат_архива_не_угадывается() {
    let room = workspace("format");
    let archive = room.join("странное.zip");
    std::fs::write(&archive, "не архив вовсе").unwrap();

    assert!(matches!(
        refusal(&archive, &room),
        ArchiveError::UnknownFormat
    ));
}

#[test]
fn имя_файла_формат_не_решает() {
    let room = workspace("misnamed");
    let archive = zipped(&room, &bundle());
    let renamed = room.join("бандл.tar.gz");
    std::fs::rename(&archive, &renamed).unwrap();

    let root = unpack(&renamed, &room.join("out"), &Limits::DEFAULT).unwrap();

    assert!(root.join("roadmap.yaml").is_file());
}

#[test]
#[cfg(unix)]
fn подменённый_каталог_уводит_распаковку_и_потому_отвергается() {
    let room = workspace("planted");
    let archive = zipped(&room, &bundle());
    let out = room.join("out");
    let side = room.join("сторона");
    std::fs::create_dir_all(&side).unwrap();
    std::fs::create_dir_all(&out).unwrap();
    std::os::unix::fs::symlink(&side, out.join("topics")).unwrap();

    let refused = unpack(&archive, &out, &Limits::DEFAULT).unwrap_err();

    assert!(
        matches!(refused, ArchiveError::Escaping { .. }),
        "{refused:?}"
    );
}

#[test]
fn объём_копится_по_всем_записям_а_не_по_последней() {
    let room = workspace("sum");
    let entries: Vec<Entry> = (0..4)
        .map(|n| Entry::file(&format!("topics/{n}.yaml"), vec![b'a'; 20]))
        .chain(std::iter::once(Entry::file("roadmap.yaml", vec![b'b'; 20])))
        .collect();
    let archive = zipped(&room, &entries);

    let refused = within(
        &archive,
        &room,
        &Limits {
            bytes: 50,
            ..Limits::DEFAULT
        },
    );

    assert!(
        matches!(refused, ArchiveError::TooBig { .. }),
        "{refused:?}"
    );
}
