use std::collections::{HashMap, hash_map::DefaultHasher};
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::io::{BufWriter, Write};
use std::os::fd::AsRawFd;
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

    fn consume(&mut self, bytes: &[u8], ends: &[usize], regions: &[u16], names: &SharedNames) {
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
        for &end in ends {
            let row = &bytes[begin..end];
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

// The input mount is immutable for the lifetime of this read-only mapping.
struct MappedInput {
    pointer: *mut std::ffi::c_void,
    length: usize,
}
unsafe extern "C" {
    fn mmap(
        address: *mut std::ffi::c_void,
        length: usize,
        protection: i32,
        flags: i32,
        fd: i32,
        offset: i64,
    ) -> *mut std::ffi::c_void;
    fn munmap(address: *mut std::ffi::c_void, length: usize) -> i32;
}
impl MappedInput {
    fn open(path: &Path) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let length = usize::try_from(file.metadata()?.len()).unwrap();
        if length == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "missing CSV header",
            ));
        }
        // Linux PROT_READ and MAP_PRIVATE; no write is made through this mapping.
        let pointer = unsafe { mmap(std::ptr::null_mut(), length, 1, 2, file.as_raw_fd(), 0) };
        if pointer as isize == -1 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(Self { pointer, length })
    }
    fn bytes(&self) -> &[u8] {
        // mmap succeeded for length bytes; this borrow cannot outlive self.
        unsafe { std::slice::from_raw_parts(self.pointer.cast(), self.length) }
    }
}
impl Drop for MappedInput {
    fn drop(&mut self) {
        unsafe {
            munmap(self.pointer, self.length);
        }
    }
}

fn scalar_quote_parity(bytes: &[u8]) -> bool {
    bytes.iter().fold(false, |odd, &byte| odd ^ (byte == b'"'))
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn vector_quote_parity(bytes: &[u8]) -> bool {
    use std::arch::x86_64::*;
    let mut offset = 0;
    let mut mask_xor = 0u32;
    while bytes.len() - offset >= 32 {
        // Each load is in bounds. XOR preserves parity across all vector blocks.
        let block = unsafe { _mm256_loadu_si256(bytes.as_ptr().add(offset).cast()) };
        mask_xor ^=
            _mm256_movemask_epi8(_mm256_cmpeq_epi8(block, _mm256_set1_epi8(b'"' as i8))) as u32;
        offset += 32;
    }
    (mask_xor.count_ones() & 1 != 0) ^ scalar_quote_parity(&bytes[offset..])
}

fn quote_parity(bytes: &[u8]) -> bool {
    #[cfg(target_arch = "x86_64")]
    if std::is_x86_feature_detected!("avx2") {
        // Runtime detection satisfies the target-feature precondition.
        return unsafe { vector_quote_parity(bytes) };
    }
    scalar_quote_parity(bytes)
}

fn following_record_end(bytes: &[u8], start: usize, mut quoted: bool) -> usize {
    let mut ends = Vec::new();
    for (block, chunk) in bytes[start..].chunks(1 << 16).enumerate() {
        ends.clear();
        record_ends(chunk, &mut quoted, &mut ends);
        if let Some(&end) = ends.first() {
            return start + (block << 16) + end;
        }
    }
    assert!(!quoted, "incomplete CSV record");
    bytes.len()
}

fn partition_records(bytes: &[u8], workers: usize) -> Vec<usize> {
    assert!(workers > 0);
    let raw: Vec<usize> = (0..=workers).map(|i| bytes.len() * i / workers).collect();
    let parity = std::thread::scope(|scope| {
        let handles: Vec<_> = raw
            .windows(2)
            .map(|pair| {
                let slice = &bytes[pair[0]..pair[1]];
                scope.spawn(move || quote_parity(slice))
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert!(
        !parity.iter().fold(false, |state, &odd| state ^ odd),
        "incomplete CSV record"
    );
    let mut cuts = vec![0];
    let mut quoted = false;
    for i in 1..workers {
        // Prefix parity gives the quote state at a raw cut, including inside an escape pair.
        quoted ^= parity[i - 1];
        cuts.push(following_record_end(bytes, raw[i], quoted));
    }
    cuts.push(bytes.len());
    cuts
}

fn consume_range(bytes: &[u8], regions: &[u16], names: &SharedNames, countries: usize) -> Partial {
    let mut partial = Partial::new(countries);
    let mut quoted = false;
    let mut ends = Vec::new();
    let mut begin = 0;
    for (block, chunk) in bytes.chunks(1 << 16).enumerate() {
        ends.clear();
        record_ends(chunk, &mut quoted, &mut ends);
        if let Some(&last) = ends.last() {
            let end = (block << 16) + last;
            for offset in &mut ends {
                *offset += (block << 16) - begin;
            }
            partial.consume(&bytes[begin..end], &ends, regions, names);
            begin = end;
        }
    }
    assert!(!quoted, "incomplete CSV record");
    if begin < bytes.len() {
        partial.consume(&bytes[begin..], &[bytes.len() - begin], regions, names);
    }
    partial
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
    let mapping = MappedInput::open(&data.join("records.csv"))?;
    let bytes = mapping.bytes();
    let header_end = bytes
        .iter()
        .position(|&b| b == b'\n')
        .expect("CSV header terminator")
        + 1;
    let header = bytes[..header_end]
        .strip_prefix(b"\xef\xbb\xbf")
        .unwrap_or(&bytes[..header_end]);
    let header = header.strip_suffix(b"\n").unwrap();
    assert_eq!(
        header.strip_suffix(b"\r").unwrap_or(header),
        b"id,name,note,region_id,amount"
    );
    let bytes = &bytes[header_end..];
    let cuts = partition_records(bytes, 6);
    let groups = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for pair in cuts.windows(2) {
            let slice = &bytes[pair[0]..pair[1]];
            let (regions, names, countries) = (&regions, &names, keys.len());
            handles.push(scope.spawn(move || consume_range(slice, regions, names, countries)));
        }
        let mut result = Partial::new(keys.len());
        for handle in handles {
            result.merge(handle.join().unwrap());
        }
        result.groups
    });
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
