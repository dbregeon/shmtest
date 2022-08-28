extern crate shmtest;

use shmtest::bench_records::LigthRecord;
use shmtest::common::stream_consumer::ShmStream;
use shmtest::common::ShmDefinition;
use std::{
    mem::size_of,
    time::{SystemTime, UNIX_EPOCH},
};

use clap::{self, Parser};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Number of warmup events
    #[clap(short, long, value_parser, default_value_t = 1000)]
    warmup_count: usize,

    /// Number of events to produce
    #[clap(short, long, value_parser, default_value_t = 100000)]
    count: usize,
}

fn main() {
    let args = Args::parse();

    test_light_load(args.warmup_count, args.count);
}

fn test_light_load(warmup_count: usize, count: usize) {
    let definition = ShmDefinition::new(
        "test_stream".to_string(),
        size_of::<LigthRecord>() * (warmup_count + count),
    );
    let mut stream: ShmStream<LigthRecord> = ShmStream::open(definition).unwrap();

    let mut sequence = 0;

    // Warmup
    while sequence < warmup_count {
        match stream.next() {
            Some(t) => {
                sequence = t.value.0;
            }
            None => {}
        }
    }

    let mut result = Vec::with_capacity(count);
    while sequence < count {
        match stream.next() {
            Some(t) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos();

                result.push((t.value.0, now, t.value.1));
                sequence = t.value.0;
            }
            None => {}
        }
    }

    let mut previous: Option<(usize, u128, u128)> = None;
    for r in result {
        println!(
            "{} {} {} {} {} {}",
            r.0,
            r.1 - r.2,
            r.1,
            r.2,
            previous.map(|p| r.1 - p.1).unwrap_or(0),
            previous.map(|p| r.2 - p.2).unwrap_or(0)
        );
        previous = Some(r);
    }

    stream.close().unwrap();
}
