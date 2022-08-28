extern crate shmtest;

use shmtest::bench_records::LigthRecord;
use shmtest::common::stream_producer::ShmStream;
use shmtest::common::ShmDefinition;
use std::{
    mem::size_of,
    time::{SystemTime, UNIX_EPOCH},
};

use clap::{self, Parser};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Interval between events
    #[clap(short, long, value_parser, default_value_t = 100)]
    beat: u64,

    /// Number of warmup events
    #[clap(short, long, value_parser, default_value_t = 1000)]
    warmup_count: usize,

    /// Number of events to produce
    #[clap(short, long, value_parser, default_value_t = 100000)]
    count: usize,
}

fn main() {
    let args = Args::parse();

    run_light_load(
        args.warmup_count,
        args.count,
        std::time::Duration::from_micros(args.beat),
    );
}

fn run_light_load(warmup_count: usize, count: usize, beat: std::time::Duration) {
    let stream_definition = ShmDefinition::new(
        "test_stream".to_string(),
        size_of::<LigthRecord>() * (warmup_count + count),
    );
    let mut stream: ShmStream<LigthRecord> = ShmStream::open(stream_definition).unwrap();

    std::thread::sleep(std::time::Duration::from_secs(30));

    // Warmup
    for i in 0..warmup_count {
        wait(beat);
        stream.insert(build_light_record(i)).unwrap();
    }

    std::thread::sleep(std::time::Duration::from_secs(5));

    // Warmup
    for i in warmup_count..count {
        wait(beat);
        stream.insert(build_light_record(i)).unwrap();
    }

    stream.close().unwrap();
}

// Thread sleep may not accomodate small durations
fn wait(duration: std::time::Duration) {
    let start = SystemTime::now();
    while start.elapsed().unwrap() < duration {
        // burn
    }
}

fn build_light_record(seq_num: usize) -> LigthRecord {
    LigthRecord {
        value: (
            seq_num + 1,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        ),
    }
}
