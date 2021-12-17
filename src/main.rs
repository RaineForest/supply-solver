use num::Rational64;
use serde::{Deserialize, Deserializer};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;

mod tree;
use crate::tree::NTree;

#[derive(Debug, Deserialize)]
struct Reagent {
    widget: String,
    quantity: u64
}

#[derive(Debug, Deserialize)]
struct Recipe {
    name: String,
    builder: String,
    // in seconds
    #[serde(deserialize_with="deserialize_decimal")]
    duration: Rational64,
    quantity: u64,
    reagents: Vec<Reagent>
}

fn deserialize_decimal<'de, D>(deserializer: D) -> Result<Rational64, D::Error> where D: Deserializer<'de> {
    Rational64::approximate_float(f64::deserialize(deserializer)?).ok_or(serde::de::Error::custom("Bad decimal"))
}

impl Recipe {
    // units/second
    pub fn rate(&self) -> Rational64 {
        Rational64::from_integer(self.quantity as i64) / self.duration
    }
}

#[derive(Debug, Deserialize)]
struct Widget {
    recipes: Vec<Recipe>
}

fn least_waste_heuristic(widget: &Widget, rate: Rational64) -> Option<(&Recipe, u64)> {
    let best_recipe = widget.recipes.iter().min_by(
        |recipe, min_rate| -> Ordering {
            let waste = (rate / recipe.rate()).fract();
            let min_waste = (rate / min_rate.rate()).fract();
            waste.cmp(&min_waste)
        }); 
    match best_recipe {
        Some(r) => {
            let frac = (rate / r.rate()).ceil();
            Some((r, (frac.numer() / frac.denom()) as u64))
        },
        None => None
    }
}

fn dep_tree<'a>(map: &'a BTreeMap<String, Widget>, widget: &String, rate: Rational64) -> NTree<(&'a Recipe, u64)> {
    let recipe = least_waste_heuristic(&map[widget], rate).unwrap();
    let mut tree = NTree::new(recipe);
    for reagent in (*tree).0.reagents.iter() {
        let requested_rate = Rational64::from_integer(reagent.quantity as i64 * recipe.1 as i64) / recipe.0.duration;
        tree.insert(dep_tree(map, &reagent.widget, requested_rate));
    }
    tree
}

fn print_tree_helper(tree: &NTree<(&Recipe, u64)>, prefix: String, is_last: bool) {
    let new_prefix = if is_last { format!("{prefix}└── ", prefix=prefix) } else { format!("{prefix}├── ", prefix=prefix) };
    println!("{prefix}{quantity}x {builder} -> {name}", prefix=new_prefix, quantity=(*tree).1, builder=(*tree).0.builder, name=(*tree).0.name);
    let children = tree.children();
    let (last, rest) = match children.split_last() {
        Some(x) => x,
        None => return
    };
    let spacer = if is_last { "    " } else { "|   " };
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

fn main() {
    let file = File::open("satisfactory.yaml").unwrap();
    let reader = BufReader::new(file);
    let map: BTreeMap<String, Widget> = serde_yaml::from_reader(reader).unwrap();

    print_tree(&dep_tree(&map, &"reinforced-iron-plate".to_owned(), Rational64::new(5, 60)));
}
