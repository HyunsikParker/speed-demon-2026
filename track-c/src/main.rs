use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

#[derive(Default)]
struct Group {
    count: u64,
    ids: u64,
    sum: i64,
    min: i64,
    max: i64,
    names: Vec<u64>,
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
    let mut groups: Vec<Group> = (0..keys.len()).map(|_| Group::default()).collect();
    let mut names = HashMap::<Vec<u8>, u32>::new();
    let mut reader = BufReader::with_capacity(1 << 20, File::open(data.join("records.csv"))?);
    let mut record = Vec::new();
    let mut name = Vec::new();
    let mut scratch = Vec::new();
    let mut quotes = false;
    let mut header = true;
    loop {
        let start = record.len();
        if reader.read_until(b'\n', &mut record)? == 0 {
            break;
        }
        for &c in &record[start..] {
            quotes ^= c == b'"';
        }
        if quotes {
            continue;
        }
        let row = record.strip_prefix(b"\xef\xbb\xbf").unwrap_or(&record);
        let row = row.strip_suffix(b"\n").unwrap_or(row);
        let row = row.strip_suffix(b"\r").unwrap_or(row);
        if header {
            assert_eq!(row, b"id,name,note,region_id,amount");
            header = false;
        } else {
            let comma = row.iter().position(|&c| c == b',').unwrap();
            let id = integer(&row[..comma]);
            let mut pos = comma + 1;
            let nb = text(row, &mut pos, &mut name, true);
            let tb = text(row, &mut pos, &mut scratch, false);
            let next = pos + row[pos..].iter().position(|&c| c == b',').unwrap();
            let rid = integer(&row[pos..next]) as usize;
            let amount = &row[next + 1..];
            let negative = amount[0] == b'-';
            let amount = if negative { &amount[1..] } else { amount };
            let dot = amount.len() - 3;
            let cents = (integer(&amount[..dot]) * 100 + integer(&amount[dot + 1..])) as i64;
            let cents = if negative { -cents } else { cents };
            let next_name = u32::try_from(names.len()).unwrap();
            let name_id = match names.get(name.as_slice()) {
                Some(&v) => v,
                None => {
                    names.insert(name.clone(), next_name);
                    next_name
                }
            };
            let g = &mut groups[usize::from(regions.get(rid).copied().unwrap_or(0))];
            if g.count == 0 {
                g.min = cents;
                g.max = cents;
            }
            g.count += 1;
            g.ids += id;
            g.sum += cents;
            g.min = g.min.min(cents);
            g.max = g.max.max(cents);
            let word = name_id as usize / 64;
            let bit = 1u64 << (name_id % 64);
            if word >= g.names.len() {
                g.names.resize(word + 1, 0);
            }
            if g.names[word] & bit == 0 {
                g.names[word] |= bit;
                g.distinct += 1;
            }
            g.name_bytes += nb;
            g.note_bytes += tb;
        }
        record.clear();
    }
    assert!(
        !quotes && record.is_empty() && !header,
        "incomplete CSV record"
    );
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
