use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::Path;

#[derive(Default)]
struct Term {
    docs: Vec<u32>,
    positions: Vec<u64>,
    phrase: bool,
}

struct Query {
    terms: Vec<usize>,
    ops: Vec<u8>,
    phrase: bool,
}

fn combine(a: &[u32], b: &[u32], op: u8) -> Vec<u32> {
    if op == b'A' && a.len().min(b.len()) < a.len().max(b.len()) / 16 {
        let (small, big) = if a.len() < b.len() { (a, b) } else { (b, a) };
        return small
            .iter()
            .copied()
            .filter(|x| big.binary_search(x).is_ok())
            .collect();
    }
    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        if a[i] < b[j] {
            if op != b'A' {
                out.push(a[i]);
            }
            i += 1;
        } else if a[i] > b[j] {
            if op == b'O' {
                out.push(b[j]);
            }
            j += 1;
        } else {
            if op != b'N' {
                out.push(a[i]);
            }
            i += 1;
            j += 1;
        }
    }
    if op != b'A' {
        out.extend_from_slice(&a[i..]);
    }
    if op == b'O' {
        out.extend_from_slice(&b[j..]);
    }
    out
}

fn answer(q: &Query, index: &[Term]) -> Vec<u32> {
    if !q.phrase {
        let mut result = index[q.terms[0]].docs.clone();
        for (&op, &term) in q.ops.iter().zip(&q.terms[1..]) {
            result = combine(&result, &index[term].docs, op);
        }
        return result;
    }
    let anchor = (0..q.terms.len())
        .min_by_key(|&i| index[q.terms[i]].positions.len())
        .unwrap();
    let mut result = Vec::new();
    for &packed in &index[q.terms[anchor]].positions {
        let doc = (packed >> 32) as u32;
        let pos = packed as u32;
        if result.last() == Some(&doc) || pos < anchor as u32 {
            continue;
        }
        let start = pos - anchor as u32;
        if q.terms.iter().enumerate().all(|(i, &t)| {
            start.checked_add(i as u32).is_some_and(|p| {
                index[t]
                    .positions
                    .binary_search(&((u64::from(doc) << 32) | u64::from(p)))
                    .is_ok()
            })
        }) {
            result.push(doc);
        }
    }
    result
}

type Summary = (usize, u64, Vec<u32>);

fn shard(
    path: &Path,
    names: &HashMap<Vec<u8>, usize>,
    phrases: &[bool],
    queries: &[Query],
    start: u64,
    end: u64,
) -> std::io::Result<Vec<Summary>> {
    let mut index: Vec<Term> = phrases
        .iter()
        .map(|&phrase| Term {
            phrase,
            ..Term::default()
        })
        .collect();
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(start.saturating_sub(1)))?;
    let mut reader = BufReader::with_capacity(1 << 20, file);
    let mut line = Vec::new();
    let mut offset = start.saturating_sub(1);
    if start != 0 {
        offset += reader.read_until(b'\n', &mut line)? as u64;
        line.clear();
    }
    let mut next_doc = None;
    while offset < end {
        let bytes = reader.read_until(b'\n', &mut line)?;
        if bytes == 0 {
            break;
        }
        offset += bytes as u64;
        let tab = line.iter().position(|&c| c == b'\t').unwrap();
        let stated_id = line[..tab]
            .iter()
            .fold(0u64, |n, &c| n * 10 + u64::from(c - b'0'));
        let doc = u32::try_from(stated_id).unwrap();
        if let Some(expected) = next_doc {
            assert_eq!(doc, expected);
        }
        next_doc = Some(doc.checked_add(1).unwrap());
        line[tab + 1..].make_ascii_lowercase();
        let mut pos = 0u32;
        let mut at = tab + 1;
        while at < line.len() {
            if !line[at].is_ascii_alphanumeric() {
                at += 1;
                continue;
            }
            let start = at;
            while at < line.len() && line[at].is_ascii_alphanumeric() {
                at += 1;
            }
            if let Some(&id) = names.get(&line[start..at]) {
                let term = &mut index[id];
                if term.docs.last() != Some(&doc) {
                    term.docs.push(doc);
                }
                if term.phrase {
                    term.positions.push((u64::from(doc) << 32) | u64::from(pos));
                }
            }
            pos = pos.checked_add(1).unwrap();
        }
        line.clear();
    }
    Ok(queries
        .iter()
        .map(|q| {
            let ids = answer(q, &index);
            (
                ids.len(),
                ids.iter().map(|&id| u64::from(id)).sum(),
                ids[..ids.len().min(10)].to_vec(),
            )
        })
        .collect())
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let data = Path::new(args.get(1).map(String::as_str).unwrap_or("/data"));
    let out = Path::new(args.get(2).map(String::as_str).unwrap_or("/out"));
    let mut names = HashMap::<Vec<u8>, usize>::new();
    let mut index = Vec::<Term>::new();
    let mut queries = Vec::new();
    let raw_queries = fs::read(data.join("queries.txt"))?;
    for row in raw_queries.split(|&c| c == b'\n').filter(|r| !r.is_empty()) {
        let phrase = row.starts_with(b"PHRASE ");
        let body = if phrase {
            &row[8..row.len() - 1]
        } else {
            &row[5..]
        };
        let mut q = Query {
            terms: Vec::new(),
            ops: Vec::new(),
            phrase,
        };
        for (i, word) in body.split(|&c| c == b' ').enumerate() {
            if !phrase && i % 2 == 1 {
                q.ops.push(word[0]);
                continue;
            }
            let id = *names.entry(word.to_vec()).or_insert_with(|| {
                index.push(Term::default());
                index.len() - 1
            });
            index[id].phrase |= phrase;
            q.terms.push(id);
        }
        queries.push(q);
    }
    let phrases: Vec<bool> = index.into_iter().map(|term| term.phrase).collect();
    let path = data.join("docs.txt");
    let size = fs::metadata(&path)?.len();
    let workers = std::env::var("SD_WORKERS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(6)
        .clamp(1, 6);
    let parts = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|i| {
                let (path, names, phrases, queries) = (&path, &names, &phrases, &queries);
                scope.spawn(move || {
                    shard(
                        path,
                        names,
                        phrases,
                        queries,
                        size * i / workers,
                        size * (i + 1) / workers,
                    )
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<std::io::Result<Vec<_>>>()
    })?;
    let mut writer = BufWriter::with_capacity(1 << 20, File::create(out.join("results.txt"))?);
    for i in 0..queries.len() {
        let count: usize = parts.iter().map(|p| p[i].0).sum();
        let sum: u64 = parts.iter().map(|p| p[i].1).sum();
        write!(writer, "{} {}", count, sum)?;
        for id in parts.iter().flat_map(|p| &p[i].2).take(10) {
            write!(writer, " {id}")?;
        }
        writeln!(writer)?;
    }
    writer.flush()
}
