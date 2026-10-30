use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fmt::Write;
use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};
use std::{env, fs, thread};

struct Numbers<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Numbers<'_> {
    fn next(&mut self) -> Option<u64> {
        while self.at < self.bytes.len() && self.bytes[self.at].is_ascii_whitespace() {
            self.at += 1;
        }
        if self.at == self.bytes.len() {
            return None;
        }
        let mut n = 0;
        while self.at < self.bytes.len() && self.bytes[self.at].is_ascii_digit() {
            n = n * 10 + u64::from(self.bytes[self.at] - b'0');
            self.at += 1;
        }
        Some(n)
    }
}

struct Graph {
    offsets: Vec<usize>,
    edges: Vec<(u32, u32)>,
    has_in: Vec<bool>,
}

impl Graph {
    fn read(path: &str) -> Self {
        let bytes = fs::read(path).expect("read graph");
        let mut input = Numbers {
            bytes: &bytes,
            at: 0,
        };
        let n = input.next().unwrap() as usize;
        let m = input.next().unwrap() as usize;
        let mut offsets = vec![0; n + 1];
        let mut has_in = vec![false; n];
        let mut triples = Vec::with_capacity(m);
        for _ in 0..m {
            let u = input.next().unwrap() as u32;
            let v = input.next().unwrap() as u32;
            let w = input.next().unwrap() as u32;
            offsets[u as usize + 1] += 1;
            has_in[v as usize] = true;
            triples.push((u, v, w));
        }
        for i in 1..=n {
            offsets[i] += offsets[i - 1];
        }
        let mut cursor = offsets[..n].to_vec();
        let mut edges = vec![(0, 0); m];
        for (u, v, w) in triples {
            edges[cursor[u as usize]] = (v, w);
            cursor[u as usize] += 1;
        }
        Self {
            offsets,
            edges,
            has_in,
        }
    }

    fn worker(&self, queries: &[(usize, usize)], next: &AtomicUsize, answers: &[AtomicI64]) {
        let mut dist = vec![0_u64; self.has_in.len()];
        let mut seen = vec![0_u32; self.has_in.len()];
        let mut epoch = 0_u32;
        let mut heap = BinaryHeap::new();
        loop {
            let index = next.fetch_add(1, Ordering::Relaxed);
            if index >= queries.len() {
                break;
            }
            let (s, t) = queries[index];
            let mut answer = -1;
            if s == t {
                answer = 0;
            } else if self.offsets[s] != self.offsets[s + 1] && self.has_in[t] {
                epoch = epoch.wrapping_add(1);
                if epoch == 0 {
                    seen.fill(0);
                    epoch = 1;
                }
                heap.clear();
                dist[s] = 0;
                seen[s] = epoch;
                heap.push(Reverse((0_u64, s)));
                while let Some(Reverse((d, u))) = heap.pop() {
                    if dist[u] != d {
                        continue;
                    }
                    if u == t {
                        answer = d as i64;
                        break;
                    }
                    for &(v, w) in &self.edges[self.offsets[u]..self.offsets[u + 1]] {
                        let v = v as usize;
                        let candidate = d + u64::from(w);
                        if seen[v] != epoch || candidate < dist[v] {
                            seen[v] = epoch;
                            dist[v] = candidate;
                            heap.push(Reverse((candidate, v)));
                        }
                    }
                }
            }
            answers[index].store(answer, Ordering::Relaxed);
        }
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let data = args.get(1).map(String::as_str).unwrap_or("/data");
    let out = args.get(2).map(String::as_str).unwrap_or("/out");
    let graph = Graph::read(&format!("{data}/graph.txt"));
    let bytes = fs::read(format!("{data}/queries.txt")).expect("read queries");
    let mut input = Numbers {
        bytes: &bytes,
        at: 0,
    };
    let mut queries = Vec::new();
    while let Some(s) = input.next() {
        queries.push((s as usize, input.next().unwrap() as usize));
    }
    let answers: Vec<_> = (0..queries.len()).map(|_| AtomicI64::new(-1)).collect();
    let next = AtomicUsize::new(0);
    let available = thread::available_parallelism().map_or(1, |n| n.get());
    let workers = env::var("SD_WORKERS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(6)
        .clamp(1, 12)
        .min(available)
        .min(queries.len().max(1));
    thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| graph.worker(&queries, &next, &answers));
        }
    });
    let mut text = String::new();
    for answer in answers {
        writeln!(text, "{}", answer.load(Ordering::Relaxed)).unwrap();
    }
    fs::write(format!("{out}/distances.txt"), text).expect("write distances");
}
