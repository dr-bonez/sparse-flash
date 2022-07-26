use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom};

fn main() {
    let app = clap::App::new("sparse-flash")
        .arg(
            clap::Arg::new("stdin-tar")
                .long("stdin-tar")
                .required_unless_present("input"),
        )
        .arg(
            clap::Arg::new("input")
                .long("input")
                .takes_value(true)
                .required_unless_present("stdin-tar"),
        )
        .arg(clap::Arg::new("destination").required(true))
        .arg(clap::Arg::new("progress").long("--progress"));
    let args = app.get_matches();
    let progress = args.contains_id("progress");
    let mut target = OpenOptions::new()
        .write(true)
        .open(args.value_of("destination").unwrap())
        .unwrap();
    if args.contains_id("stdin-tar") {
        let mut input = std::io::stdin();
        let mut start = 0;
        loop {
            let mut sections = Vec::<(u64, u64)>::new();
            let mut archive = tar::Archive::new(&mut input);
            if let Some(mut entry) = archive.entries().unwrap().next().map(|e| e.unwrap()) {
                entry
                    .header()
                    .as_ustar()
                    .expect("must use POSIX compliant TAR");
                let length = entry.size();
                if !entry
                    .pax_extensions()
                    .unwrap()
                    .into_iter()
                    .flat_map(|e| e.into_iter().map(|e| e.unwrap()))
                    .any(|e| e.key().unwrap() == "GNU.sparse.major")
                {}
                drop(entry);
                let mut ctr = 0;
                let mut line = String::new();
                ctr += input.read_line(&mut line).unwrap();
                let count: usize = line.trim().parse().unwrap();
                let mut size_on_disk = 0;
                for _ in 0..count {
                    line.truncate(0);
                    ctr += input.read_line(&mut line).unwrap();
                    let offset = line.trim().parse().unwrap();
                    line.truncate(0);
                    ctr += input.read_line(&mut line).unwrap();
                    let length = line.trim().parse().unwrap();
                    sections.push((offset, length));
                    size_on_disk += length;
                }
                input.read_exact(&mut vec![0; 512 - (ctr % 512)]).unwrap();
                let progress = if progress {
                    Some(
                        indicatif::ProgressBar::new(size_on_disk).with_style(
                            indicatif::ProgressStyle::default_bar()
                                .template("{bytes} {elapsed_precise} [{binary_bytes_per_sec}] [{wide_bar}] {percent}% ETA {eta_precise}")
                                .progress_chars("=> "),
                        ),
                    )
                } else {
                    None
                };
                for (offset, length) in sections {
                    target.seek(SeekFrom::Start(start + offset)).unwrap();
                    if let Some(progress) = &progress {
                        std::io::copy(
                            &mut progress.wrap_read((&mut input).take(length)),
                            &mut target,
                        )
                        .unwrap();
                    } else {
                        std::io::copy(&mut (&mut input).take(length), &mut target).unwrap();
                    }
                }
                start += length;
                if let Some(progress) = &progress {
                    progress.finish();
                }
            } else {
                break;
            }
        }
    } else if let Some(input_path) = args.value_of("input") {
        todo!()
    }
    if progress {
        let progress = indicatif::ProgressBar::new_spinner().with_message("Syncing Data");
        progress.enable_steady_tick(300);
        target.sync_all().unwrap();
        progress.finish();
    } else {
        target.sync_all().unwrap();
    }
}
