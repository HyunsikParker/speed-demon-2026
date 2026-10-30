use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::Path;

#[derive(Default)]
struct Term {
    docs: Vec<u32>,
    needed: bool,
}

struct Query {
    terms: Vec<usize>,
    ops: Vec<u8>,
    phrase: bool,
    pattern: usize,
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
    let mut result = index[q.terms[0]].docs.clone();
    for (&op, &term) in q.ops.iter().zip(&q.terms[1..]) {
        result = combine(&result, &index[term].docs, op);
    }
    result
}

#[derive(Default)]
struct Node {
    next: Vec<(usize, usize)>,
    fail: usize,
    output: Vec<usize>,
}

struct Automaton {
    nodes: Vec<Node>,
    root: Vec<usize>,
    patterns: usize,
}

impl Automaton {
    fn new(queries: &mut [Query], terms: usize) -> Self {
        let mut ac = Self {
            nodes: vec![Node::default()],
            root: vec![0; terms],
            patterns: 0,
        };
        let mut edges = HashMap::new();
        for q in queries.iter_mut().filter(|q| q.phrase) {
            let mut state = 0;
            for &term in &q.terms {
                state = *edges.entry((state, term)).or_insert_with(|| {
                    let child = ac.nodes.len();
                    ac.nodes.push(Node::default());
                    ac.nodes[state].next.push((term, child));
                    child
                });
            }
            if ac.nodes[state].output.is_empty() {
                ac.nodes[state].output.push(ac.patterns);
                ac.patterns += 1;
            }
            q.pattern = ac.nodes[state].output[0];
        }
        drop(edges);
        for node in &mut ac.nodes {
            node.next.sort_unstable();
        }
        let mut queue = std::collections::VecDeque::new();
        for &(term, child) in &ac.nodes[0].next {
            ac.root[term] = child;
            queue.push_back(child);
        }
        while let Some(state) = queue.pop_front() {
            for (term, child) in ac.nodes[state].next.clone() {
                let fail = ac.step(ac.nodes[state].fail, term);
                ac.nodes[child].fail = fail;
                let output = ac.nodes[fail].output.clone();
                ac.nodes[child].output.extend(output);
                queue.push_back(child);
            }
        }
        ac
    }

    fn step(&self, mut state: usize, term: usize) -> usize {
        loop {
            if state == 0 {
                return self.root[term];
            }
            if let Ok(i) = self.nodes[state]
                .next
                .binary_search_by_key(&term, |&(t, _)| t)
            {
                return self.nodes[state].next[i].1;
            }
            state = self.nodes[state].fail;
        }
    }
}

type Summary = (usize, u64, Vec<u32>);

fn shard(
    path: &Path,
    names: &HashMap<Vec<u8>, usize>,
    needed: &[bool],
    queries: &[Query],
    ac: &Automaton,
    start: u64,
    end: u64,
) -> std::io::Result<Vec<Summary>> {
    let mut index: Vec<Term> = needed
        .iter()
        .map(|&needed| Term {
            needed,
            ..Term::default()
        })
        .collect();
    let mut matches = vec![Vec::<u32>::new(); ac.patterns];
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
        let mut state = 0;
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
                if term.needed && term.docs.last() != Some(&doc) {
                    term.docs.push(doc);
                }
                state = ac.step(state, id);
                for &pattern in &ac.nodes[state].output {
                    if matches[pattern].last() != Some(&doc) {
                        matches[pattern].push(doc);
                    }
                }
            } else {
                state = 0;
            }
        }
        line.clear();
    }
    Ok(queries
        .iter()
        .map(|q| {
            let ids = if q.phrase {
                matches[q.pattern].clone()
            } else {
                answer(q, &index)
            };
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
            pattern: usize::MAX,
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
            index[id].needed |= !phrase;
            q.terms.push(id);
        }
        queries.push(q);
    }
    let needed: Vec<bool> = index.into_iter().map(|term| term.needed).collect();
    let ac = Automaton::new(&mut queries, names.len());
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
                let (path, names, needed, queries, ac) = (&path, &names, &needed, &queries, &ac);
                scope.spawn(move || {
                    shard(
                        path,
                        names,
                        needed,
                        queries,
                        ac,
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
