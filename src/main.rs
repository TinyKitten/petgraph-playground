extern crate csv;

use std::collections::BinaryHeap;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::process;
use std::{collections::HashMap, env};

use csv::StringRecord;
use petgraph::visit::EdgeRef;
use petgraph::{
    graph::{NodeIndex, UnGraph},
    Graph, Undirected,
};

fn get_first_arg() -> Result<OsString, Box<dyn Error>> {
    match env::args_os().nth(1) {
        None => Err(From::from("expected 1 argument, but got none")),
        Some(file_path) => Ok(file_path),
    }
}

fn fopen(path: &OsString) -> Result<File, Box<dyn Error>> {
    let file = File::open(path)?;
    Ok(file)
}

fn csv_parse(file: File) -> Result<Vec<StringRecord>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_reader(file);
    let records: Vec<StringRecord> = rdr.records().map(|rec| rec.unwrap()).collect();
    Ok(records)
}

fn dijkstra_with_path(
    graph: &Graph<i32, f64, Undirected>,
    start: NodeIndex,
) -> (
    HashMap<NodeIndex, f64>,
    HashMap<NodeIndex, Option<NodeIndex>>,
) {
    let mut dist_map = HashMap::new();
    let mut prev_map = HashMap::new();

    for node in graph.node_indices() {
        dist_map.insert(node, f64::INFINITY);
        prev_map.insert(node, None);
    }
    *dist_map.get_mut(&start).unwrap() = 0.0;

    let mut heap = BinaryHeap::new();
    heap.push(State {
        cost: 0.0,
        node: start,
    });

    while let Some(State { cost, node }) = heap.pop() {
        if cost > dist_map[&node] {
            continue;
        }
        for edge in graph.edges(node) {
            let next = edge.target();
            let next_cost = cost + edge.weight();

            if next_cost < dist_map[&next] {
                *dist_map.get_mut(&next).unwrap() = next_cost;
                *prev_map.get_mut(&next).unwrap() = Some(node);
                heap.push(State {
                    cost: next_cost,
                    node: next,
                });
            }
        }
    }

    (dist_map, prev_map)
}

#[derive(Copy, Clone, PartialEq)]
struct State {
    cost: f64,
    node: NodeIndex,
}

impl Eq for State {}

impl Ord for State {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn reconstruct_path(
    prev_map: &HashMap<NodeIndex, Option<NodeIndex>>,
    start: NodeIndex,
    goal: NodeIndex,
) -> Option<Vec<NodeIndex>> {
    let mut path = Vec::new();
    let mut current = goal;
    while let Some(&Some(prev)) = prev_map.get(&current) {
        path.push(current);
        current = prev;

        if current == start {
            path.push(start);
            path.reverse();
            return Some(path);
        }
    }
    None
}

fn main() {
    let file_path = match get_first_arg() {
        Ok(file_path) => file_path,
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    };

    let records = csv_parse(fopen(&file_path).unwrap()).unwrap();

    let edges = records.iter().map(|rec| {
        let start = rec.get(0).unwrap().parse::<u32>().unwrap();
        let goal = rec.get(1).unwrap().parse::<u32>().unwrap();
        let weight = rec.get(2).unwrap().parse::<f64>().unwrap();
        (start, goal, weight)
    });

    let graph = UnGraph::<i32, f64>::from_edges(edges);

    let start_id = 1;
    let goal_id = 100;

    let (dist_map, prev_map) = dijkstra_with_path(&graph, start_id.into());

    println!("start -> goal の最短距離: {}m", dist_map[&goal_id.into()]);

    if let Some(path) = reconstruct_path(&prev_map, start_id.into(), goal_id.into()) {
        println!("start -> goal の最短距離 の経路: {:?}", path);
    } else {
        println!("や、経路のNASA🚀");
    }
}
