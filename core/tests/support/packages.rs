use tolearn_core::package::{MANIFEST, SCHEMA};

use super::pictures::UNSAFE;
use super::sealing::{content, linked, loose, manifest, sealed, tarred, with, without, zipped};

pub const CHIPTUNE: &str = "examples/chiptune";
pub const NES_DEV: &str = "fixtures/v2/valid/nes-dev";
pub const DIAGRAM: &str = "assets/voices.svg";

pub struct Broken {
    pub code: &'static str,
    pub name: &'static str,
    pub names: &'static str,
    pub bytes: Vec<u8>,
}

pub fn corpus() -> Vec<Broken> {
    let base = content(CHIPTUNE);
    let listed = manifest(SCHEMA, &base);
    let mut changed = base.clone();
    for (name, data) in &mut changed {
        if name == "program.yaml" {
            data.extend_from_slice(b"\n");
        }
    }
    let mut corpus = vec![
        broken(
            "archive.unknown-format",
            "not-an-archive",
            "not-an-archive.tolearn",
            b"program.yaml\n".to_vec(),
        ),
        broken(
            "archive.unknown-format",
            "tarball",
            "tarball.tolearn",
            tarred(&with(base.clone(), MANIFEST, &listed)),
        ),
        broken(
            "archive.escaping-path",
            "entry-climbs-out",
            "../outside.yaml",
            zipped(&with(base.clone(), "../outside.yaml", b"x")),
        ),
        broken(
            "archive.link",
            "entry-is-a-link",
            "assets/link.svg",
            linked(&base, "assets/link.svg", "/etc/passwd"),
        ),
        broken(
            "package.no-manifest",
            "no-manifest",
            MANIFEST,
            zipped(&base),
        ),
        broken(
            "package.bad-manifest",
            "manifest-not-json",
            MANIFEST,
            zipped(&with(base.clone(), MANIFEST, b"{\"schema\": ")),
        ),
        broken(
            "package.bad-manifest",
            "files-a-list",
            MANIFEST,
            zipped(&with(
                base.clone(),
                MANIFEST,
                br#"{"schema": "tolearn/package/1", "files": []}"#,
            )),
        ),
        broken(
            "package.bad-manifest",
            "manifest-a-directory",
            MANIFEST,
            zipped(&with(base.clone(), "manifest.json/inside", b"x")),
        ),
        broken(
            "package.version",
            "next-major",
            MANIFEST,
            zipped(&with(
                base.clone(),
                MANIFEST,
                &manifest("tolearn/package/2", &base),
            )),
        ),
        broken(
            "package.unlisted",
            "file-outside-the-manifest",
            "notes.txt",
            zipped(&with(
                with(base.clone(), MANIFEST, &listed),
                "notes.txt",
                b"x",
            )),
        ),
        broken(
            "package.missing",
            "listed-file-absent",
            "assets/pulse-wave.png",
            zipped(&with(
                without(base.clone(), "assets/pulse-wave.png"),
                MANIFEST,
                &listed,
            )),
        ),
        broken(
            "package.checksum",
            "program-changed-after-sealing",
            "program.yaml",
            zipped(&with(changed, MANIFEST, &listed)),
        ),
        broken(
            "package.unused",
            "progress-rides-along",
            "state/progress.yaml",
            sealed(with(base.clone(), "state/progress.yaml", b"stages: {}\n")),
        ),
        broken(
            "library.invalid",
            "diagram-without-its-svg",
            "pipeline.svg",
            sealed(loose(
                "fixtures/v2/invalid/program.missing-asset__diagram-without-its-svg",
            )),
        ),
        broken(
            "yaml.wrong-type",
            "stage-title-a-list",
            "stages/linker.yaml",
            sealed(loose(
                "fixtures/v2/invalid/yaml.wrong-type__stage-title-a-list",
            )),
        ),
    ];
    for (name, document) in UNSAFE {
        corpus.push(broken(
            "package.unsafe-svg",
            name,
            DIAGRAM,
            sealed(with(base.clone(), DIAGRAM, document.as_bytes())),
        ));
    }
    corpus
}

fn broken(code: &'static str, name: &'static str, names: &'static str, bytes: Vec<u8>) -> Broken {
    Broken {
        code,
        name,
        names,
        bytes,
    }
}
