//! DiskDome command line: scan benchmarks and exports.
//!
//! Every engine change is measured here first (time and memory on a real volume).
//!
//! ```text
//! diskdome-cli mft-read C: [--threads N] [--chunk-mb M]      read the $MFT only (admin)
//! diskdome-cli scan C: [options] [--export FILE.csv]         read, parse, build, sum (admin)
//! diskdome-cli scan --dump FILE [options]                    the same from an $MFT dump
//! diskdome-cli scan --synthetic RECORDS [options]            the same on a generated drive
//! diskdome-cli mft-dump C: FILE                              save the $MFT to a file (admin)
//! diskdome-cli compare OURS.csv WIZTREE.csv [--top N]        compare folder totals
//! diskdome-cli inspect --dump FILE --path "C:\dir" [--top N]  a folder's children and their records
//! ```

mod compare;
mod export;
mod inspect;

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use diskdome_scan_ntfs::dump::{DumpSource, write_dump};
use diskdome_scan_ntfs::synthetic::SyntheticSource;
use diskdome_scan_ntfs::{Options, ScanError, Source, Volume, read_only, scan};

const USAGE: &str = "\
usage:
  diskdome-cli mft-read C: [--threads N] [--chunk-mb M]
  diskdome-cli scan (C: | --dump FILE | --synthetic RECORDS) [--threads N] [--chunk-mb M]
                    [--export FILE.csv] [--drive X]
  diskdome-cli mft-dump C: FILE
  diskdome-cli compare OURS.csv WIZTREE.csv [--top N]
  diskdome-cli inspect --dump FILE --path FOLDER [--top N]

mft-read, scan C: and mft-dump read the volume directly: run them as administrator.";

struct Args {
    positional: Vec<String>,
    threads: Option<usize>,
    chunk_mb: Option<usize>,
    export: Option<PathBuf>,
    dump: Option<PathBuf>,
    synthetic: Option<u64>,
    drive: Option<char>,
    path: Option<String>,
    top: usize,
}

fn parse_args(raw: &[String]) -> Result<Args, String> {
    let mut args = Args {
        positional: Vec::new(),
        threads: None,
        chunk_mb: None,
        export: None,
        dump: None,
        synthetic: None,
        drive: None,
        path: None,
        top: 20,
    };
    let mut it = raw.iter();
    while let Some(a) = it.next() {
        let mut value = |name: &str| {
            it.next()
                .cloned()
                .ok_or_else(|| format!("{name} needs a value"))
        };
        let number = |name: &str, v: String| {
            v.replace('_', "")
                .parse::<u64>()
                .map_err(|_| format!("{name}: not a number: {v}"))
        };
        match a.as_str() {
            "--threads" => args.threads = Some(number(a, value(a)?)? as usize),
            "--chunk-mb" => args.chunk_mb = Some(number(a, value(a)?)? as usize),
            "--export" => args.export = Some(PathBuf::from(value(a)?)),
            "--dump" => args.dump = Some(PathBuf::from(value(a)?)),
            "--synthetic" => args.synthetic = Some(number(a, value(a)?)?),
            "--top" => args.top = number(a, value(a)?)? as usize,
            "--drive" => args.drive = drive_letter(&value(a)?),
            "--path" => args.path = Some(value(a)?),
            other if other.starts_with("--") => return Err(format!("unknown option {other}")),
            other => args.positional.push(other.to_string()),
        }
    }
    Ok(args)
}

fn drive_letter(s: &str) -> Option<char> {
    let c = s.chars().next()?.to_ascii_uppercase();
    c.is_ascii_alphabetic().then_some(c)
}

fn options(args: &Args) -> Options {
    let mut o = Options::default();
    if let Some(t) = args.threads {
        o.threads = t.max(1);
    }
    if let Some(mb) = args.chunk_mb {
        o.chunk_bytes = mb.max(1) << 20;
    }
    o
}

fn mb(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

fn secs(d: Duration) -> f64 {
    d.as_secs_f64()
}

fn print_memory(label: &str) {
    match diskdome_platform::memory() {
        Some(m) => println!(
            "{label}: working set {:.0} MB (peak {:.0} MB), private {:.0} MB (peak {:.0} MB)",
            mb(m.working_set),
            mb(m.peak_working_set),
            mb(m.private),
            mb(m.peak_private)
        ),
        None => println!("{label}: not measured on this platform"),
    }
}

fn open_volume(args: &Args) -> Result<(Volume, char), String> {
    let letter = args
        .positional
        .get(1)
        .and_then(|s| drive_letter(s))
        .ok_or("which drive? e.g. C:")?;
    let volume = Volume::open(letter).map_err(|e| e.to_string())?;
    let s = volume.summary();
    println!(
        "volume {letter}: cluster {} B, record {} B, $MFT {:.0} MB in {} extents (from {})",
        s.bytes_per_cluster,
        s.record_size,
        mb(s.mft_bytes),
        s.extents,
        if s.from_retrieval_pointers {
            "FSCTL_GET_RETRIEVAL_POINTERS"
        } else {
            "record 0"
        }
    );
    Ok((volume, letter))
}

fn cmd_mft_read(args: &Args) -> Result<(), String> {
    let (volume, _) = open_volume(args)?;
    let opts = options(args);
    let r = read_only(&volume, &opts).map_err(|e| e.to_string())?;
    println!(
        "read {:.0} MB in {:.2} s: {:.0} MB/s ({} threads, {} MB chunks)",
        mb(r.bytes),
        secs(r.wall),
        mb(r.bytes) / secs(r.wall).max(1e-9),
        r.threads,
        r.chunk_bytes >> 20
    );
    print_memory("memory");
    Ok(())
}

fn run_scan(source: &impl Source, args: &Args, drive: char) -> Result<(), String> {
    let opts = options(args);
    let total = Instant::now();
    let (tree, r) = scan(source, &opts).map_err(|e: ScanError| e.to_string())?;
    let wall = total.elapsed();
    println!(
        "records {} ({:.0} MB of $MFT), {} threads, {} KB chunks",
        r.records,
        mb(r.mft_bytes),
        r.threads,
        r.chunk_bytes >> 10
    );
    println!(
        "entries {} = {} files + {} folders; {} extension records, {} extra hard links, {} unreachable",
        r.tree.linked,
        r.tree.files,
        r.tree.dirs,
        r.extension_records,
        r.extra_links,
        r.tree.unreachable
    );
    println!(
        "time {:.2} s = read+parse {:.2} s + merge {:.2} s + totals {:.2} s ({:.0} MB/s)",
        secs(wall),
        secs(r.read_parse),
        secs(r.merge),
        secs(r.finish),
        mb(r.mft_bytes) / secs(r.read_parse).max(1e-9)
    );
    println!(
        "model {:.0} MB = {:.1} bytes per entry (names {:.0} MB)",
        mb(tree.heap_bytes() as u64),
        tree.heap_bytes() as f64 / f64::from(r.tree.linked.max(1)),
        mb(tree.names.heap_bytes() as u64)
    );
    println!(
        "root: size {:.3} GB, allocated {:.3} GB",
        tree.size[tree.root as usize] as f64 / 1e9,
        tree.alloc_bytes(tree.root) as f64 / 1e9
    );
    print_memory("memory after the scan");
    if let Some(path) = &args.export {
        let start = Instant::now();
        let folders = export::folders_csv(&tree, drive, path).map_err(|e| e.to_string())?;
        println!(
            "exported {folders} folders to {} in {:.2} s",
            path.display(),
            secs(start.elapsed())
        );
    }
    Ok(())
}

fn cmd_scan(args: &Args) -> Result<(), String> {
    if let Some(path) = &args.dump {
        let source = DumpSource::open(path.clone()).map_err(|e| e.to_string())?;
        return run_scan(&source, args, args.drive.unwrap_or('C'));
    }
    if let Some(records) = args.synthetic {
        let source = SyntheticSource {
            records,
            record_size: 1024,
        };
        if args.threads.is_some() || args.chunk_mb.is_some() {
            // Generation alone, to subtract from the scan time.
            let r = read_only(&source, &options(args)).map_err(|e| e.to_string())?;
            println!(
                "generating the records alone: {:.2} s (part of read+parse below)",
                secs(r.wall)
            );
        }
        return run_scan(&source, args, args.drive.unwrap_or('S'));
    }
    let (volume, letter) = open_volume(args)?;
    run_scan(&volume, args, letter)
}

fn cmd_inspect(args: &Args) -> Result<(), String> {
    let dump = args.dump.as_ref().ok_or("inspect needs --dump FILE")?;
    let path = args.path.as_deref().ok_or("inspect needs --path FOLDER")?;
    let source = DumpSource::open(dump.clone()).map_err(|e| e.to_string())?;
    let (tree, report) = scan(&source, &options(args)).map_err(|e| e.to_string())?;
    inspect::run(&tree, &source, report.records, path, args.top)
}

fn cmd_mft_dump(args: &Args) -> Result<(), String> {
    let (volume, _) = open_volume(args)?;
    let path = args.positional.get(2).ok_or("where to? a file path")?;
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut out = std::io::BufWriter::with_capacity(1 << 20, file);
    let start = Instant::now();
    let bytes = write_dump(&volume, &mut out).map_err(|e| e.to_string())?;
    println!(
        "saved {:.0} MB to {path} in {:.1} s",
        mb(bytes),
        secs(start.elapsed())
    );
    Ok(())
}

fn main() -> ExitCode {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let args = match parse_args(&raw) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}\n\n{USAGE}");
            return ExitCode::FAILURE;
        }
    };
    let result = match args.positional.first().map(String::as_str) {
        Some("mft-read") => cmd_mft_read(&args),
        Some("scan") => cmd_scan(&args),
        Some("mft-dump") => cmd_mft_dump(&args),
        Some("inspect") => cmd_inspect(&args),
        Some("compare") => compare::run(&args.positional[1..], args.top),
        Some(_) | None => {
            println!("diskdome-cli {}\n\n{USAGE}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
