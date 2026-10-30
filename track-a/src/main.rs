use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::thread;

#[derive(Default)]
struct Group {
    latencies: Vec<u32>,
    errors: u64,
}

#[derive(Default)]
struct Aggregate {
    hours: Vec<[u8; 13]>,
    hour_ids: HashMap<[u8; 13], usize>,
    endpoints: Vec<Vec<u8>>,
    endpoint_ids: HashMap<Vec<u8>, usize>,
    groups: Vec<Vec<Group>>,
}
impl Aggregate {
    fn hour(&mut self, hour: [u8; 13]) -> usize {
        let h = *self.hour_ids.entry(hour).or_insert_with(|| {
            self.hours.push(hour);
            self.hours.len() - 1
        });
        assert!(h < 72);
        h
    }
    fn endpoint(&mut self, endpoint: &[u8]) -> usize {
        match self.endpoint_ids.get(endpoint) {
            Some(&id) => id,
            None => {
                let id = self.endpoints.len();
                self.endpoints.push(endpoint.to_vec());
                self.endpoint_ids.insert(endpoint.to_vec(), id);
                self.groups.push(Vec::new());
                id
            }
        }
    }
    fn group(&mut self, endpoint: usize, hour: usize) -> &mut Group {
        let groups = &mut self.groups[endpoint];
        if groups.len() <= hour {
            groups.resize_with(hour + 1, Group::default);
        }
        &mut groups[hour]
    }
    fn add(&mut self, line: &[u8]) {
        // Fixed-width UTC timestamps make the first 13 bytes the exact hour key.
        let hour: [u8; 13] = line[..13].try_into().unwrap();
        let h = self.hour(hour);
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
        let e = self.endpoint(endpoint);
        let mut latency = 0u32;
        for &digit in &line[cursor + 5..line.len() - 1] {
            latency = latency * 10 + u32::from(digit - b'0');
        }
        let group = self.group(e, h);
        group.latencies.push(latency);
        group.errors += u64::from(line[cursor + 1] == b'5');
    }
    fn merge(&mut self, other: Self) {
        let hours: Vec<_> = other.hours.into_iter().map(|h| self.hour(h)).collect();
        for (endpoint, groups) in other.endpoints.into_iter().zip(other.groups) {
            let e = self.endpoint(&endpoint);
            for (h, mut source) in groups.into_iter().take(hours.len()).enumerate() {
                if source.latencies.is_empty() {
                    continue;
                }
                let dest = self.group(e, hours[h]);
                dest.errors += source.errors;
                if dest.latencies.is_empty() {
                    dest.latencies = source.latencies;
                } else {
                    dest.latencies.append(&mut source.latencies);
                }
            }
        }
    }
    fn write_to(&mut self, output: &mut impl Write) -> io::Result<()> {
        let mut hour_order: Vec<usize> = (0..self.hours.len()).collect();
        hour_order.sort_unstable_by_key(|&h| self.hours[h]);
        let mut endpoint_order: Vec<usize> = (0..self.endpoints.len()).collect();
        endpoint_order.sort_unstable_by(|&a, &b| self.endpoints[a].cmp(&self.endpoints[b]));
        for h in hour_order {
            for &e in &endpoint_order {
                let Some(group) = self.groups[e].get_mut(h) else {
                    continue;
                };
                let n = group.latencies.len();
                if n == 0 {
                    continue;
                }
                group.latencies.sort_unstable();
                let percentile = |p: usize| group.latencies[(p * n).div_ceil(100) - 1];
                writeln!(
                    output,
                    "{}:00:00Z {} {} {} {} {} {} {}",
                    std::str::from_utf8(&self.hours[h]).unwrap(),
                    std::str::from_utf8(&self.endpoints[e]).unwrap(),
                    n,
                    percentile(50),
                    percentile(95),
                    percentile(99),
                    group.errors,
                    group.errors * 1_000_000 / n as u64
                )?;
            }
        }
        Ok(())
    }
}

fn read_range(path: &Path, start: u64, end: u64) -> io::Result<Aggregate> {
    let mut file = File::open(path)?;
    let mut position = start.saturating_sub(1);
    file.seek(SeekFrom::Start(position))?;
    let mut input = BufReader::with_capacity(1 << 20, file);
    let mut line = Vec::with_capacity(256);
    if start != 0 {
        position += input.read_until(b'\n', &mut line)? as u64;
        line.clear();
    }
    let mut aggregate = Aggregate::default();
    while position < end {
        let bytes = input.read_until(b'\n', &mut line)?;
        if bytes == 0 {
            break;
        }
        position += bytes as u64;
        aggregate.add(&line);
        line.clear();
    }
    Ok(aggregate)
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let data = PathBuf::from(args.get(1).map(String::as_str).unwrap_or("/data"));
    let out = PathBuf::from(args.get(2).map(String::as_str).unwrap_or("/out"));
    let path = data.join("requests.log");
    let size = path.metadata()?.len();
    let available = thread::available_parallelism().map_or(1, |n| n.get());
    let workers = std::env::var("SD_WORKERS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(6)
        .clamp(1, 12)
        .min(available);
    let shards = thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|i| {
                let path = &path;
                let start = size * i as u64 / workers as u64;
                let end = size * (i + 1) as u64 / workers as u64;
                scope.spawn(move || read_range(path, start, end))
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().expect("reader worker"))
            .collect::<io::Result<Vec<_>>>()
    })?;
    let mut shards = shards.into_iter();
    let mut aggregate = shards.next().unwrap();
    for shard in shards {
        aggregate.merge(shard);
    }
    let mut output = BufWriter::with_capacity(1 << 20, File::create(out.join("summary.txt"))?);
    aggregate.write_to(&mut output)?;
    output.flush()
}
