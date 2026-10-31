use std::collections::HashMap;
use std::fs::File;
use std::hash::{BuildHasherDefault, Hasher};
use std::io::{self, BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::thread;

#[derive(Default)]
struct IdentityHasher(u64);
impl Hasher for IdentityHasher {
    fn write(&mut self, _: &[u8]) { unreachable!("u64 fingerprint keys only") }
    fn write_u64(&mut self, value: u64) { self.0 = value; }
    fn finish(&self) -> u64 { self.0 }
}

fn endpoint_key(bytes: &[u8]) -> (usize, u64) {
    let mut pos = 0;
    let mut hash = 0xcbf29ce484222325u64;
    while pos + 8 <= bytes.len() {
        let word = u64::from_le_bytes(bytes[pos..pos + 8].try_into().unwrap());
        let spaces = word ^ 0x2020202020202020;
        if spaces.wrapping_sub(0x0101010101010101) & !spaces & 0x8080808080808080 != 0 { break; }
        hash = (hash ^ word).wrapping_mul(0x100000001b3);
        pos += 8;
    }
    let mut tail = 0u64;
    let start = pos;
    while pos < bytes.len() && bytes[pos] != b' ' {
        tail |= u64::from(bytes[pos]) << (8 * (pos - start));
        pos += 1;
    }
    if pos != start { hash = (hash ^ tail).wrapping_mul(0x100000001b3); }
    (pos, hash.rotate_left(17) ^ pos as u64)
}

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
    endpoint_ids: HashMap<u64, Vec<usize>, BuildHasherDefault<IdentityHasher>>,
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
    fn endpoint(&mut self, endpoint: &[u8], hash: u64) -> usize {
        let bucket = self.endpoint_ids.entry(hash).or_default();
        for &id in &*bucket {
            if self.endpoints[id] == endpoint { return id; }
        }
        let id = self.endpoints.len();
        self.endpoints.push(endpoint.to_vec());
        self.groups.push(Vec::new());
        bucket.push(id);
        id
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
        let (length, fingerprint) = endpoint_key(&line[start..]);
        cursor = start + length;
        let e = self.endpoint(&line[start..cursor], fingerprint);
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
            let e = self.endpoint(&endpoint, endpoint_key(&endpoint).1);
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
        let mut entries: Vec<_> = self.endpoints.iter().zip(std::mem::take(&mut self.groups)).collect();
        entries.sort_unstable_by(|a, b| a.0.cmp(b.0));
        let chunk_size = entries.len().div_ceil(worker_count()).max(1);
        let hours = &self.hours;
        thread::scope(|scope| {
            let mut receivers = Vec::new();
            let mut handles = Vec::new();
            for chunk in entries.chunks_mut(chunk_size) {
                let (sender, receiver) = std::sync::mpsc::sync_channel(0);
                receivers.push(receiver);
                let order = &hour_order;
                handles.push(scope.spawn(move || -> io::Result<()> {
                    for &h in order {
                        let mut buffer = Vec::new();
                        for (endpoint, groups) in &mut *chunk {
                            let Some(group) = groups.get_mut(h) else { continue; };
                            let n = group.latencies.len();
                            if n == 0 { continue; }
                            group.latencies.sort_unstable();
                            let percentile = |p: usize| group.latencies[(p * n).div_ceil(100) - 1];
                            writeln!(
                                buffer,
                                "{}:00:00Z {} {} {} {} {} {} {}",
                                std::str::from_utf8(&hours[h]).unwrap(),
                                std::str::from_utf8(endpoint).unwrap(),
                                n, percentile(50), percentile(95), percentile(99),
                                group.errors, group.errors * 1_000_000 / n as u64
                            )?;
                            group.latencies = Vec::new();
                        }
                        // A rendezvous bounds each worker to its current hour buffer.
                        if sender.send(buffer).is_err() { return Ok(()); }
                    }
                    Ok(())
                }));
            }
            for _ in &hour_order {
                for receiver in &receivers {
                    output.write_all(&receiver.recv().expect("output worker"))?;
                }
            }
            for handle in handles { handle.join().expect("output worker")?; }
            Ok(())
        })
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

fn worker_count() -> usize {
    let available = thread::available_parallelism().map_or(1, |n| n.get());
    std::env::var("SD_WORKERS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(6)
        .clamp(1, 12)
        .min(available)
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let data = PathBuf::from(args.get(1).map(String::as_str).unwrap_or("/data"));
    let out = PathBuf::from(args.get(2).map(String::as_str).unwrap_or("/out"));
    let path = data.join("requests.log");
    let size = path.metadata()?.len();
    let workers = worker_count();
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
