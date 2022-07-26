use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};

use tar::GnuSparseHeader;

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
        .arg(clap::Arg::new("destination").required(true));
    let args = app.get_matches();
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
            let entry = archive.entries().unwrap().next().unwrap().unwrap();
            entry
                .header()
                .as_ustar()
                .expect("must use POSIX compliant TAR");
            let length = entry.size();
            drop(entry);
            let mut ctr = 0;
            let mut line = String::new();
            ctr += input.read_line(&mut line).unwrap();
            let count: usize = line.trim().parse().unwrap();
            for _ in 0..count {
                line.truncate(0);
                ctr += input.read_line(&mut line).unwrap();
                let offset = line.trim().parse().unwrap();
                line.truncate(0);
                ctr += input.read_line(&mut line).unwrap();
                let length = line.trim().parse().unwrap();
                sections.push((offset, length));
            }
            input.read_exact(&mut vec![0; 512 - (ctr % 512)]).unwrap();
            for (offset, length) in dbg!(sections) {
                target.seek(SeekFrom::Start(start + offset)).unwrap();
                std::io::copy(&mut (&mut input).take(length), &mut target).unwrap();
            }
            start += length;
        }
    } else if let Some(input_path) = args.value_of("input") {
        todo!()
    }
}
