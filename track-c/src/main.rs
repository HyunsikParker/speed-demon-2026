use std::collections::{HashMap, hash_map::DefaultHasher};
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::sync::Mutex;

#[derive(Default)]
struct Group {
    count: u64,
    ids: u64,
    sum: i64,
    min: i64,
    max: i64,
    distinct: u64,
    name_bytes: u64,
    note_bytes: u64,
}

fn integer(b: &[u8]) -> u64 {
    b.iter().fold(0, |n, &c| n * 10 + u64::from(c - b'0'))
}

// Decode a text field, or count its decoded bytes without retaining the text.
fn text(b: &[u8], pos: &mut usize, out: &mut Vec<u8>, keep: bool) -> u64 {
    out.clear();
    let mut len = 0;
    let quoted = b[*pos] == b'"';
    if quoted {
        *pos += 1;
    }
    loop {
        let c = b[*pos];
        if quoted && c == b'"' {
            *pos += 1;
            if b.get(*pos) != Some(&b'"') {
                break;
            }
        } else if !quoted && c == b',' {
            break;
        }
        if keep {
            out.push(c);
        }
        len += 1;
        *pos += 1;
    }
    assert_eq!(b[*pos], b',');
    *pos += 1;
    len
}

fn money(n: i64) -> String {
    let v = n.unsigned_abs();
    format!("{}{}.{:02}", if n < 0 { "-" } else { "" }, v / 100, v % 100)
}

fn scalar_record_ends(bytes: &[u8], quoted: &mut bool, ends: &mut Vec<usize>, base: usize) {
    for (i, &byte) in bytes.iter().enumerate() {
        *quoted ^= byte == b'"';
        if byte == b'\n' && !*quoted {
            ends.push(base + i + 1);
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn vector_record_ends(bytes: &[u8], quoted: &mut bool, ends: &mut Vec<usize>) {
    use std::arch::x86_64::*;
    let mut offset = 0;
    while bytes.len() - offset >= 32 {
        // The loop bounds guarantee a full 32-byte load; alignment is not required.
        let block = unsafe { _mm256_loadu_si256(bytes.as_ptr().add(offset).cast()) };
        let quotes =
            _mm256_movemask_epi8(_mm256_cmpeq_epi8(block, _mm256_set1_epi8(b'"' as i8))) as u32;
        let lines =
            _mm256_movemask_epi8(_mm256_cmpeq_epi8(block, _mm256_set1_epi8(b'\n' as i8))) as u32;
        let mut parity = quotes;
        parity ^= parity << 1;
        parity ^= parity << 2;
        parity ^= parity << 4;
        parity ^= parity << 8;
        parity ^= parity << 16;
        let mut outside = lines & if *quoted { parity } else { !parity };
        while outside != 0 {
            ends.push(offset + outside.trailing_zeros() as usize + 1);
            outside &= outside - 1;
        }
        *quoted ^= quotes.count_ones() & 1 != 0;
        offset += 32;
    }
    scalar_record_ends(&bytes[offset..], quoted, ends, offset);
}

fn record_ends(bytes: &[u8], quoted: &mut bool, ends: &mut Vec<usize>) {
    #[cfg(target_arch = "x86_64")]
    if std::is_x86_feature_detected!("avx2") {
        // Runtime feature detection satisfies vector_record_ends' CPU precondition.
        unsafe { vector_record_ends(bytes, quoted, ends) };
        return;
    }
    scalar_record_ends(bytes, quoted, ends, 0);
}

#[derive(Default)]
struct Batch {
    bytes: Vec<u8>,
    ends: Vec<usize>,
}

type NameMap = HashMap<Vec<u8>, Vec<u64>>;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct InlineName {
    words: [u64; 2],
    len: u8,
}

#[derive(Clone)]
enum NameRef {
    Inline(InlineName),
    Arena(std::ops::Range<usize>),
}

#[derive(Default)]
struct NameShard {
    inline: HashMap<InlineName, [u64; 4]>,
    other: NameMap,
}

struct SharedNames {
    parts: Vec<Mutex<NameShard>>,
    words: usize,
}

impl SharedNames {
    fn new(countries: usize) -> Self {
        Self {
            parts: (0..64).map(|_| Mutex::new(NameShard::default())).collect(),
            words: countries.div_ceil(64),
        }
    }

    fn part(&self, name: &[u8]) -> usize {
        let mut hash = DefaultHasher::new();
        name.hash(&mut hash);
        hash.finish() as usize % self.parts.len()
    }

    fn reference(&self, name: &[u8], arena: &mut Vec<u8>) -> NameRef {
        if self.words <= 4 && name.len() <= 16 {
            let mut bytes = [0u8; 16];
            bytes[..name.len()].copy_from_slice(name);
            NameRef::Inline(InlineName {
                words: [
                    u64::from_le_bytes(bytes[..8].try_into().unwrap()),
                    u64::from_le_bytes(bytes[8..].try_into().unwrap()),
                ],
                len: name.len() as u8,
            })
        } else {
            let begin = arena.len();
            arena.extend_from_slice(name);
            NameRef::Arena(begin..arena.len())
        }
    }

    fn apply(&self, bytes: &[u8], requests: &[Vec<(NameRef, usize)>], groups: &mut [Group]) {
        assert_eq!(requests.len(), self.parts.len());
        for (part, requests) in self.parts.iter().zip(requests) {
            if requests.is_empty() {
                continue;
            }
            let mut names = part.lock().unwrap();
            for (reference, country) in requests {
                let country = *country;
                let bit = 1u64 << (country % 64);
                let fresh = match reference {
                    NameRef::Inline(key) => {
                        let bits = names.inline.entry(*key).or_default();
                        let fresh = bits[country / 64] & bit == 0;
                        bits[country / 64] |= bit;
                        fresh
                    }
                    NameRef::Arena(range) => {
                        let name = &bytes[range.clone()];
                        if let Some(bits) = names.other.get_mut(name) {
                            let fresh = bits[country / 64] & bit == 0;
                            bits[country / 64] |= bit;
                            fresh
                        } else {
                            let mut bits = vec![0; self.words];
                            bits[country / 64] = bit;
                            names.other.insert(name.to_vec(), bits);
                            true
                        }
                    }
                };
                groups[country].distinct += u64::from(fresh);
            }
        }
    }
}

#[derive(Default)]
struct Partial {
    groups: Vec<Group>,
    name: Vec<u8>,
    scratch: Vec<u8>,
    decoded: Vec<u8>,
    requests: Vec<Vec<(NameRef, usize)>>,
}

impl Partial {
    fn new(countries: usize) -> Self {
        Self {
            groups: (0..countries).map(|_| Group::default()).collect(),
            ..Self::default()
        }
    }

    fn consume(&mut self, batch: Batch, regions: &[u16], names: &SharedNames) {
        let mut begin = 0;
        let Self {
            groups,
            name,
            scratch,
            decoded,
            requests,
        } = self;
        decoded.clear();
        requests.resize_with(names.parts.len(), Vec::new);
        for shard in requests.iter_mut() {
            shard.clear();
        }
        for end in batch.ends {
            let row = &batch.bytes[begin..end];
            begin = end;
            let row = row.strip_prefix(b"\xef\xbb\xbf").unwrap_or(row);
            let row = row.strip_suffix(b"\n").unwrap_or(row);
            let row = row.strip_suffix(b"\r").unwrap_or(row);
            let comma = row.iter().position(|&c| c == b',').unwrap();
            let id = integer(&row[..comma]);
            let mut pos = comma + 1;
            let nb = text(row, &mut pos, name, true);
            let tb = text(row, &mut pos, scratch, false);
            let next = pos + row[pos..].iter().position(|&c| c == b',').unwrap();
            let rid = integer(&row[pos..next]) as usize;
            let amount = &row[next + 1..];
            let negative = amount[0] == b'-';
            let amount = if negative { &amount[1..] } else { amount };
            let dot = amount.len() - 3;
            let cents = (integer(&amount[..dot]) * 100 + integer(&amount[dot + 1..])) as i64;
            let cents = if negative { -cents } else { cents };
            let country = usize::from(regions.get(rid).copied().unwrap_or(0));
            let reference = names.reference(name, decoded);
            requests[names.part(name)].push((reference, country));
            let g = &mut groups[country];
            if g.count == 0 {
                g.min = cents;
                g.max = cents;
            }
            g.count += 1;
            g.ids += id;
            g.sum += cents;
            g.min = g.min.min(cents);
            g.max = g.max.max(cents);
            g.name_bytes += nb;
            g.note_bytes += tb;
        }
        names.apply(decoded, requests, groups);
    }

    fn merge(&mut self, other: Self) {
        for (g, h) in self.groups.iter_mut().zip(other.groups) {
            if h.count == 0 {
                continue;
            }
            if g.count == 0 {
                g.min = h.min;
                g.max = h.max;
            }
            g.distinct += h.distinct;
            g.count += h.count;
            g.ids += h.ids;
            g.sum += h.sum;
            g.min = g.min.min(h.min);
            g.max = g.max.max(h.max);
            g.name_bytes += h.name_bytes;
            g.note_bytes += h.note_bytes;
        }
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let data = Path::new(args.get(1).map(String::as_str).unwrap_or("/data"));
    let out = Path::new(args.get(2).map(String::as_str).unwrap_or("/out"));
    let mut keys = vec!["_UNMATCHED".to_owned()];
    let mut codes = HashMap::<Vec<u8>, u16>::new();
    let mut regions = vec![0u16];
    let lookup = fs::read(data.join("regions.csv"))?;
    for row in lookup
        .split(|&c| c == b'\n')
        .skip(1)
        .filter(|r| !r.is_empty())
    {
        let comma = row.iter().position(|&c| c == b',').unwrap();
        let id = integer(&row[..comma]) as usize;
        let code = &row[comma + 1..];
        let group = *codes.entry(code.to_vec()).or_insert_with(|| {
            let id = u16::try_from(keys.len()).unwrap();
            keys.push(String::from_utf8(code.to_vec()).unwrap());
            id
        });
        regions.resize(regions.len().max(id + 1), 0);
        regions[id] = group;
    }
    let names = SharedNames::new(keys.len());
    let groups = std::thread::scope(|scope| -> std::io::Result<Vec<Group>> {
        let workers = 6;
        let mut senders = Vec::new();
        let mut handles = Vec::new();
        for _ in 0..workers {
            let (sender, receiver) = std::sync::mpsc::sync_channel::<Batch>(1);
            let regions = &regions;
            let countries = keys.len();
            let names = &names;
            senders.push(sender);
            handles.push(scope.spawn(move || {
                let mut partial = Partial::new(countries);
                for batch in receiver {
                    partial.consume(batch, regions, names);
                }
                partial
            }));
        }
        let mut reader = BufReader::with_capacity(1 << 20, File::open(data.join("records.csv"))?);
        let mut batch = Batch::default();
        let mut quotes = false;
        let mut header = true;
        let mut worker = 0;
        let mut chunk_ends = Vec::new();
        loop {
            let chunk = reader.fill_buf()?;
            if chunk.is_empty() {
                break;
            }
            chunk_ends.clear();
            record_ends(chunk, &mut quotes, &mut chunk_ends);
            let mut copied = 0;
            for &end in &chunk_ends {
                if header {
                    batch.bytes.extend_from_slice(&chunk[copied..end]);
                    copied = end;
                    let row = batch
                        .bytes
                        .strip_prefix(b"\xef\xbb\xbf")
                        .unwrap_or(&batch.bytes);
                    let row = row.strip_suffix(b"\n").unwrap_or(row);
                    let row = row.strip_suffix(b"\r").unwrap_or(row);
                    assert_eq!(row, b"id,name,note,region_id,amount");
                    header = false;
                    batch.bytes.clear();
                } else {
                    let logical_end = batch.bytes.len() + end - copied;
                    batch.ends.push(logical_end);
                    if logical_end >= 1 << 16 {
                        batch.bytes.extend_from_slice(&chunk[copied..end]);
                        copied = end;
                        senders[worker].send(std::mem::take(&mut batch)).unwrap();
                        worker = (worker + 1) % workers;
                    }
                }
            }
            batch.bytes.extend_from_slice(&chunk[copied..]);
            let consumed = chunk.len();
            reader.consume(consumed);
        }
        assert!(!quotes && !header, "incomplete CSV record");
        if !batch.bytes.is_empty() {
            if batch.ends.last() != Some(&batch.bytes.len()) {
                batch.ends.push(batch.bytes.len());
            }
            senders[worker].send(batch).unwrap();
        }
        drop(senders);
        let mut result = Partial::new(keys.len());
        for handle in handles {
            result.merge(handle.join().unwrap());
        }
        Ok(result.groups)
    })?;
    let mut order: Vec<usize> = (0..keys.len()).collect();
    order.sort_unstable_by(|&a, &b| keys[a].cmp(&keys[b]));
    let mut writer = BufWriter::new(File::create(out.join("aggregates.txt"))?);
    for i in order {
        let g = &groups[i];
        if g.count == 0 {
            continue;
        }
        writeln!(
            writer,
            "{} {} {} {} {} {} {} {} {}",
            keys[i],
            g.count,
            g.ids,
            money(g.sum),
            money(g.min),
            money(g.max),
            g.distinct,
            g.name_bytes,
            g.note_bytes
        )?;
    }
    writer.flush()
}
