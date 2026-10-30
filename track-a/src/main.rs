use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

#[derive(Default)]
struct Group {
    latencies: Vec<u32>,
    errors: u64,
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let data = PathBuf::from(args.get(1).map(String::as_str).unwrap_or("/data"));
    let out = PathBuf::from(args.get(2).map(String::as_str).unwrap_or("/out"));
    let mut input = BufReader::with_capacity(1 << 20, File::open(data.join("requests.log"))?);
    let mut hours = Vec::<[u8; 13]>::new();
    let mut hour_ids = HashMap::<[u8; 13], usize>::new();
    let mut endpoints = Vec::<Vec<u8>>::new();
    let mut endpoint_ids = HashMap::<Vec<u8>, usize>::new();
    let mut groups = Vec::<Vec<Group>>::new();
    let mut line = Vec::with_capacity(256);

    while input.read_until(b'\n', &mut line)? != 0 {
        // Fixed-width UTC timestamps make the first 13 bytes the exact hour key.
        let hour: [u8; 13] = line[..13].try_into().unwrap();
        let h = *hour_ids.entry(hour).or_insert_with(|| {
            hours.push(hour);
            hours.len() - 1
        });
        // The contract covers at most 72 hours, regardless of input order.
        assert!(h < 72);
        let mut cursor = 28;
        while line[cursor] != b' ' {
            cursor += 1;
        }
        let start = cursor + 1;
        cursor = start;
        while line[cursor] != b' ' {
            cursor += 1;
        }
        let endpoint = &line[start..cursor];
        let e = match endpoint_ids.get(endpoint) {
            Some(&id) => id,
            None => {
                let id = endpoints.len();
                endpoints.push(endpoint.to_vec());
                endpoint_ids.insert(endpoint.to_vec(), id);
                groups.push((0..72).map(|_| Group::default()).collect());
                id
            }
        };
        let mut latency = 0u32;
        for &digit in &line[cursor + 5..line.len() - 1] {
            latency = latency * 10 + u32::from(digit - b'0');
        }
        let group = &mut groups[e][h];
        group.latencies.push(latency);
        group.errors += u64::from(line[cursor + 1] == b'5');
        line.clear();
    }

    let mut hour_order: Vec<usize> = (0..hours.len()).collect();
    hour_order.sort_unstable_by_key(|&h| hours[h]);
    let mut endpoint_order: Vec<usize> = (0..endpoints.len()).collect();
    endpoint_order.sort_unstable_by(|&a, &b| endpoints[a].cmp(&endpoints[b]));
    let mut output = BufWriter::with_capacity(1 << 20, File::create(out.join("summary.txt"))?);
    for h in hour_order {
        for &e in &endpoint_order {
            let group = &mut groups[e][h];
            let n = group.latencies.len();
            if n == 0 {
                continue;
            }
            group.latencies.sort_unstable();
            let percentile = |p: usize| group.latencies[(p * n).div_ceil(100) - 1];
            writeln!(
                output,
                "{}:00:00Z {} {} {} {} {} {} {}",
                std::str::from_utf8(&hours[h]).unwrap(),
                std::str::from_utf8(&endpoints[e]).unwrap(),
                n,
                percentile(50),
                percentile(95),
                percentile(99),
                group.errors,
                group.errors * 1_000_000 / n as u64
            )?;
        }
    }
    output.flush()
}
