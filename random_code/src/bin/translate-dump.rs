use std::io::{stdin, Read};

use arbitrary::{Arbitrary, Unstructured};
use random_code::Node;

fn main() {
    let buf = stdin()
        .lock()
        .bytes()
        .collect::<Result<Vec<_>, _>>()
        .expect("could not read input");
    let mut u = Unstructured::new(&buf);
    let node = Node::arbitrary(&mut u).expect("could not parse input");
    println!("{node}");
}
