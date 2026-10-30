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
}
impl Graph {
    fn build(n: usize, triples: &[(u32, u32, u32)], reverse: bool) -> Self {
        let mut offsets = vec![0; n + 1];
        for &(u, v, _) in triples {
            offsets[if reverse { v } else { u } as usize + 1] += 1;
        }
        for i in 1..=n {
            offsets[i] += offsets[i - 1];
        }
        let mut cursor = offsets[..n].to_vec();
        let mut edges = vec![(0, 0); triples.len()];
        for &(u, v, w) in triples {
            let (a, b) = if reverse { (v, u) } else { (u, v) };
            edges[cursor[a as usize]] = (b, w);
            cursor[a as usize] += 1;
        }
        Self { offsets, edges }
    }
    fn read(path: &str) -> [Self; 2] {
        let bytes = fs::read(path).expect("read graph");
        let mut input = Numbers {
            bytes: &bytes,
            at: 0,
        };
        let n = input.next().unwrap() as usize;
        let m = input.next().unwrap() as usize;
        let mut triples = Vec::with_capacity(m);
        for _ in 0..m {
            triples.push((
                input.next().unwrap() as u32,
                input.next().unwrap() as u32,
                input.next().unwrap() as u32,
            ));
        }
        [
            Self::build(n, &triples, false),
            Self::build(n, &triples, true),
        ]
    }
}

struct Search {
    dist: Vec<u64>,
    touched: Vec<usize>,
    heap: BinaryHeap<Reverse<(u64, usize)>>,
}
impl Search {
    fn new(n: usize) -> Self {
        Self {
            dist: vec![u64::MAX; n],
            touched: Vec::new(),
            heap: BinaryHeap::new(),
        }
    }
    fn start(&mut self, source: usize) {
        for u in self.touched.drain(..) {
            self.dist[u] = u64::MAX;
        }
        self.heap.clear();
        self.dist[source] = 0;
        self.touched.push(source);
        self.heap.push(Reverse((0, source)));
    }
    fn minimum(&mut self) -> Option<u64> {
        while let Some(&Reverse((d, u))) = self.heap.peek() {
            if d == self.dist[u] {
                return Some(d);
            }
            self.heap.pop();
        }
        None
    }
    fn expand(&mut self, graph: &Graph, other: &Self, best: &mut u64) {
        let Reverse((d, u)) = self.heap.pop().unwrap();
        if other.dist[u] != u64::MAX {
            *best = (*best).min(d + other.dist[u]);
        }
        for &(v, w) in &graph.edges[graph.offsets[u]..graph.offsets[u + 1]] {
            let v = v as usize;
            let next = d + u64::from(w);
            if next < self.dist[v] {
                if self.dist[v] == u64::MAX {
                    self.touched.push(v);
                }
                self.dist[v] = next;
                self.heap.push(Reverse((next, v)));
                if other.dist[v] != u64::MAX {
                    *best = (*best).min(next + other.dist[v]);
                }
            }
        }
    }
}

fn worker(
    graphs: &[Graph; 2],
    queries: &[(usize, usize)],
    next: &AtomicUsize,
    answers: &[AtomicI64],
) {
    let n = graphs[0].offsets.len() - 1;
    let (mut forward, mut backward) = (Search::new(n), Search::new(n));
    loop {
        let index = next.fetch_add(1, Ordering::Relaxed);
        if index >= queries.len() {
            break;
        }
        let (s, t) = queries[index];
        let mut best = u64::MAX;
        if s == t {
            best = 0;
        } else if graphs[0].offsets[s] != graphs[0].offsets[s + 1]
            && graphs[1].offsets[t] != graphs[1].offsets[t + 1]
        {
            forward.start(s);
            backward.start(t);
            let mut turn = false;
            while let (Some(a), Some(b)) = (forward.minimum(), backward.minimum()) {
                if a.saturating_add(b) >= best {
                    break;
                }
                if turn {
                    backward.expand(&graphs[1], &forward, &mut best);
                } else {
                    forward.expand(&graphs[0], &backward, &mut best);
                }
                turn = !turn;
            }
        }
        answers[index].store(
            if best == u64::MAX { -1 } else { best as i64 },
            Ordering::Relaxed,
        );
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let data = args.get(1).map(String::as_str).unwrap_or("/data");
    let out = args.get(2).map(String::as_str).unwrap_or("/out");
    let graphs = Graph::read(&format!("{data}/graph.txt"));
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
            scope.spawn(|| worker(&graphs, &queries, &next, &answers));
        }
    });
    let mut text = String::new();
    for answer in answers {
        writeln!(text, "{}", answer.load(Ordering::Relaxed)).unwrap();
    }
    fs::write(format!("{out}/distances.txt"), text).expect("write distances");
}
