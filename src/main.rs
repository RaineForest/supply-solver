use num::Rational64;
use serde::{Deserialize, Deserializer};
use std::cmp::Ordering;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use structopt::StructOpt;

mod tree;
use crate::tree::NTree;

mod hypergraph;
use crate::hypergraph::Hypergraph;

#[derive(Clone, Debug, Deserialize)]
struct Reagent {
    widget: String,
    quantity: u64
}

#[derive(Clone, Debug, Deserialize)]
struct Recipe {
    name: String,
    builder: String,
    // in seconds
    #[serde(deserialize_with="deserialize_decimal")]
    duration: Rational64,
    products: Vec<Reagent>,
    reagents: Vec<Reagent>
}

fn deserialize_decimal<'de, D>(deserializer: D) -> Result<Rational64, D::Error> where D: Deserializer<'de> {
    Rational64::approximate_float(f64::deserialize(deserializer)?).ok_or(serde::de::Error::custom("Bad decimal"))
}

impl Recipe {
    // units/second
    pub fn rate(&self, widget: &String) -> Rational64 {
        let reagent = self.products.iter().filter(| r | widget == &r.widget).next().unwrap();
        Rational64::from_integer(reagent.quantity as i64) / self.duration
    }
}

#[derive(Debug, Deserialize)]
struct Cookbook {
    widgets: Vec<String>,
    recipes: Vec<Recipe>
}

impl Cookbook {
    pub fn parse(file_path: &PathBuf) -> Self {
        let file = File::open(file_path).unwrap();
        let reader = BufReader::new(file);
        serde_yaml::from_reader(reader).unwrap()
    }
}

fn least_waste_heuristic<'a>(graph: &'a Hypergraph<String, Recipe>, widget: &String, rate: Rational64) -> Option<(&'a Recipe, u64)> {
    let best_recipe = graph.neighbor_of(widget).unwrap().iter().map(| e | graph.get_weight(e).unwrap()).min_by(
        |recipe, min_rate| -> Ordering {
            let waste = (rate / recipe.rate(widget)).fract();
            let min_waste = (rate / min_rate.rate(widget)).fract();
            waste.cmp(&min_waste)
        }); 
    match best_recipe {
        Some(r) => {
            let frac = (rate / r.rate(widget)).ceil();
            Some((r, (frac.numer() / frac.denom()) as u64))
        },
        None => None
    }
}

fn dep_tree<'a>(graph: &'a Hypergraph<String, Recipe>, widget: &String, rate: Rational64) -> NTree<(&'a Recipe, u64)> {
    let recipe = least_waste_heuristic(graph, widget, rate).unwrap();
    let mut tree = NTree::new(recipe);
    for reagent in (*tree).0.reagents.iter() {
        let requested_rate = Rational64::from_integer(reagent.quantity as i64 * recipe.1 as i64) / recipe.0.duration;
        tree.insert(dep_tree(graph, &reagent.widget, requested_rate));
    }
    tree
}

fn print_tree_helper(tree: &NTree<(&Recipe, u64)>, prefix: String, is_last: bool) {
    let new_prefix = if is_last { format!("{prefix}????????? ", prefix=prefix) } else { format!("{prefix}????????? ", prefix=prefix) };
    println!("{prefix}{quantity}x {builder} -> {name}", prefix=new_prefix, quantity=(*tree).1, builder=(*tree).0.builder, name=(*tree).0.name);
    let children = tree.children();
    let (last, rest) = match children.split_last() {
        Some(x) => x,
        None => return
    };
    let spacer = if is_last { "    " } else { "???   " };
    for child in rest.iter() {
        print_tree_helper(child, format!("{prefix}{spacer}", prefix=prefix, spacer=spacer), false);
    }
    print_tree_helper(last,format!("{prefix}{spacer}", prefix=prefix, spacer=spacer), true);
}

fn print_tree(tree: &NTree<(&Recipe, u64)>) {
    println!("{quantity}x {builder} -> {name}", quantity=(*tree).1, builder=(*tree).0.builder, name=(*tree).0.name);
    let children = tree.children();
    let (last, rest) = match children.split_last() {
        Some(x) => x,
        None => return
    };
    for child in rest.iter() {
        print_tree_helper(child, "".to_owned(), false);
    }
    print_tree_helper(last, "".to_owned(), true);
}

#[derive(StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    game_def: std::path::PathBuf,

    widget: String,
    rate: f64
}

fn main() {
    let args = Cli::from_args();
    let cookbook = Cookbook::parse(&args.game_def);
    let mut graph = Hypergraph::<String, Recipe>::new();
    for widget in cookbook.widgets {
        graph.insert_node(widget.clone());
    }
    for recipe in cookbook.recipes {
        let sources: Vec<String> = recipe.reagents.iter().map(| r | r.widget.clone()).collect();
        let destinations: Vec<String> = recipe.products.iter().map(| r | r.widget.clone()).collect();
        graph.insert_edge(&sources, &destinations, recipe.clone());
    }

    print_tree(&dep_tree(&graph, &args.widget, Rational64::approximate_float(args.rate).unwrap()));
}
