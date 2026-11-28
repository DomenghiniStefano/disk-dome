//! A file's category from its name (`docs/design-system.md` § 3.2): the twelve families that colour the map, the side
//! panel and the file icons, and that the saved scan summaries total. The lower-case extension decides, except for a
//! few names that win over it (the page, hibernation and swap files, the NTFS metafiles: System). Folders have no
//! category. The names match Slint's `FileCategory` (`crates/dome-ui/ui/tokens/data.slint`), so Rust and the
//! interface say the same thing.
//!
//! No allocation per name: the extension is lower-cased into a small buffer on the stack and matched as bytes.

/// The twelve categories, in the order of the spec's table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FileCategory {
    Video,
    Image,
    Audio,
    Document,
    Game,
    App,
    Data,
    Code,
    Build,
    Archive,
    System,
    Other,
}

impl FileCategory {
    pub const ALL: [FileCategory; 12] = [
        FileCategory::Video,
        FileCategory::Image,
        FileCategory::Audio,
        FileCategory::Document,
        FileCategory::Game,
        FileCategory::App,
        FileCategory::Data,
        FileCategory::Code,
        FileCategory::Build,
        FileCategory::Archive,
        FileCategory::System,
        FileCategory::Other,
    ];

    /// The name the spec's table and Slint's enum give it.
    pub fn name(self) -> &'static str {
        match self {
            FileCategory::Video => "video",
            FileCategory::Image => "image",
            FileCategory::Audio => "audio",
            FileCategory::Document => "document",
            FileCategory::Game => "game",
            FileCategory::App => "app",
            FileCategory::Data => "data",
            FileCategory::Code => "code",
            FileCategory::Build => "build",
            FileCategory::Archive => "archive",
            FileCategory::System => "system",
            FileCategory::Other => "other",
        }
    }
}

/// The longest extension the table knows (`uasset`, `rmeta`, `sqlite`): a longer one is `Other` without looking.
const LONGEST_EXTENSION: usize = 6;

/// Names that win over their extension: Windows' own large files and the NTFS metafiles (§ 3.2).
const SYSTEM_NAMES: [&str; 17] = [
    "pagefile.sys",
    "hiberfil.sys",
    "swapfile.sys",
    "$mft",
    "$mftmirr",
    "$logfile",
    "$volume",
    "$attrdef",
    "$bitmap",
    "$boot",
    "$badclus",
    "$secure",
    "$upcase",
    "$extend",
    "$quota",
    "$usnjrnl",
    "$objid",
];

/// The category of a file called `name` (the name only, not the path).
pub fn category_of(name: &str) -> FileCategory {
    if SYSTEM_NAMES.iter().any(|n| name.eq_ignore_ascii_case(n)) {
        return FileCategory::System;
    }
    let Some(dot) = name.rfind('.') else {
        return FileCategory::Other;
    };
    let extension = &name.as_bytes()[dot + 1..];
    if extension.is_empty() || extension.len() > LONGEST_EXTENSION {
        return FileCategory::Other;
    }
    let mut lower = [0u8; LONGEST_EXTENSION];
    for (to, from) in lower.iter_mut().zip(extension) {
        *to = from.to_ascii_lowercase();
    }
    by_extension(&lower[..extension.len()])
}

/// The spec's table (§ 3.2, first version). `ts` is TypeScript here, not an MPEG transport stream: on a developer's
/// drive the many `.ts` files are `node_modules` sources (both lists of the table name it).
fn by_extension(extension: &[u8]) -> FileCategory {
    match extension {
        b"mp4" | b"mkv" | b"mov" | b"avi" | b"wmv" | b"webm" | b"m4v" | b"mpg" | b"flv" => {
            FileCategory::Video
        }
        b"jpg" | b"jpeg" | b"png" | b"gif" | b"webp" | b"heic" | b"avif" | b"tif" | b"tiff"
        | b"bmp" | b"psd" | b"raw" | b"cr2" | b"nef" | b"arw" | b"dng" | b"svg" | b"ico" => {
            FileCategory::Image
        }
        b"mp3" | b"flac" | b"wav" | b"aac" | b"ogg" | b"opus" | b"m4a" | b"wma" | b"aiff" => {
            FileCategory::Audio
        }
        b"pdf" | b"doc" | b"docx" | b"xls" | b"xlsx" | b"ppt" | b"pptx" | b"odt" | b"ods"
        | b"odp" | b"txt" | b"md" | b"rtf" | b"epub" | b"csv" => FileCategory::Document,
        b"pak" | b"vpk" | b"bundle" | b"assets" | b"uasset" | b"ucas" | b"utoc" | b"bsa"
        | b"ba2" | b"forge" | b"wad" => FileCategory::Game,
        b"exe" | b"dll" | b"msi" | b"msix" | b"appx" | b"sys" | b"drv" | b"ocx" | b"efi"
        | b"so" | b"dylib" => FileCategory::App,
        b"db" | b"sqlite" | b"mdf" | b"ldf" | b"pst" | b"ost" | b"dat" | b"bin" | b"log"
        | b"dmp" | b"bak" | b"tmp" | b"cache" | b"etl" | b"evtx" => FileCategory::Data,
        b"js" | b"mjs" | b"cjs" | b"ts" | b"tsx" | b"jsx" | b"json" | b"map" | b"rs" | b"py"
        | b"c" | b"cpp" | b"h" | b"cs" | b"java" | b"go" | b"rb" | b"php" | b"css" | b"scss"
        | b"html" | b"vue" | b"toml" | b"yaml" | b"lock" => FileCategory::Code,
        b"o" | b"obj" | b"rlib" | b"rmeta" | b"pdb" | b"lib" | b"a" | b"d" | b"class" | b"pyc"
        | b"jar" | b"wasm" | b"node" => FileCategory::Build,
        b"zip" | b"7z" | b"rar" | b"tar" | b"gz" | b"xz" | b"zst" | b"bz2" | b"cab" | b"iso"
        | b"img" | b"vhd" | b"vhdx" | b"vmdk" | b"wim" | b"esd" => FileCategory::Archive,
        _ => FileCategory::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::{FileCategory, LONGEST_EXTENSION, category_of};

    /// The spec's table (§ 3.2): each category's enum name and its extensions, read from `docs/design-system.md`.
    fn spec_table() -> Vec<(String, Vec<String>)> {
        let doc = include_str!("../../../docs/design-system.md");
        doc.lines()
            .filter(|l| l.starts_with("| ") && l.matches('|').count() == 10)
            .filter_map(|l| {
                let cells: Vec<&str> = l.trim_matches('|').split('|').map(str::trim).collect();
                let name = cells
                    .get(1)?
                    .strip_prefix('`')?
                    .strip_suffix('`')?
                    .to_owned();
                let extensions = cells
                    .get(8)?
                    .split_whitespace()
                    .map(str::to_owned)
                    .collect();
                Some((name, extensions))
            })
            .collect()
    }

    #[test]
    fn every_extension_of_the_spec_maps_to_its_category() {
        let table = spec_table();
        assert_eq!(table.len(), FileCategory::ALL.len(), "one row per category");
        for (name, extensions) in &table {
            let category = FileCategory::ALL
                .into_iter()
                .find(|c| c.name() == name)
                .unwrap_or_else(|| panic!("no category {name}"));
            for extension in extensions {
                // `ts` is in two rows of the table; it is TypeScript (see `by_extension`).
                if category == FileCategory::Video && extension == "ts" {
                    continue;
                }
                // System and Other list names and prose, not extensions.
                if matches!(category, FileCategory::System | FileCategory::Other) {
                    continue;
                }
                assert_eq!(
                    category_of(&format!("file.{extension}")),
                    category,
                    ".{extension}"
                );
                assert!(
                    extension.len() <= LONGEST_EXTENSION,
                    ".{extension} is longer than the buffer"
                );
            }
        }
    }

    #[test]
    fn names_win_over_the_extension() {
        assert_eq!(category_of("pagefile.sys"), FileCategory::System);
        assert_eq!(category_of("HIBERFIL.SYS"), FileCategory::System);
        assert_eq!(category_of("$MFT"), FileCategory::System);
        assert_eq!(
            category_of("driver.sys"),
            FileCategory::App,
            "another .sys is a program"
        );
    }

    #[test]
    fn case_dots_and_the_unknown() {
        assert_eq!(category_of("Movie.MKV"), FileCategory::Video);
        assert_eq!(
            category_of("archive.tar.gz"),
            FileCategory::Archive,
            "the last extension decides"
        );
        assert_eq!(category_of("index.ts"), FileCategory::Code);
        assert_eq!(category_of("README"), FileCategory::Other, "no extension");
        assert_eq!(category_of("trailing."), FileCategory::Other);
        assert_eq!(category_of(".gitignore"), FileCategory::Other);
        assert_eq!(
            category_of("file.unknownext"),
            FileCategory::Other,
            "longer than any known"
        );
        assert_eq!(category_of("file.xyz"), FileCategory::Other);
    }
}
