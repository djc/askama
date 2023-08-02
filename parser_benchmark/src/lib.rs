#![deny(elided_lifetimes_in_paths)]
#![deny(unreachable_pub)]
#![allow(clippy::type_complexity)]

mod expr;
mod node;
mod strings;

use arbitrary::{Arbitrary, Unstructured};
use getrandom::getrandom;
use rand::{RngCore, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use node::Node;

trait ToSource {
    fn write_into(&self, buf: &mut String);
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Seed {
    Full([u8; 32]),
    Partial(u64),
    #[default]
    Random,
}

impl From<[u8; 32]> for Seed {
    fn from(seed: [u8; 32]) -> Self {
        Self::Full(seed)
    }
}

impl From<u64> for Seed {
    fn from(seed: u64) -> Self {
        Self::Partial(seed)
    }
}

impl Seed {
    pub fn into_rng(self) -> Xoshiro256PlusPlus {
        Xoshiro256PlusPlus::from_seed(match self {
            Self::Full(v) => v,
            Self::Partial(partial) => {
                let p = partial.to_ne_bytes();
                // front part of SHA-256's IV
                [
                    0x42, 0x8a, 0x2f, 0x98, 0x71, 0x37, 0x44, 0x91, 0xb5, 0xc0, 0xfb, 0xcf, 0xe9,
                    0xb5, 0xdb, 0xa5, 0x39, 0x56, 0xc2, 0x5b, 0x59, 0xf1, 0x11, 0xf1, p[0], p[1],
                    p[2], p[3], p[4], p[5], p[6], p[7],
                ]
            }
            Self::Random => {
                let mut seed = [0; 32];
                getrandom(&mut seed).expect("should be able to get random bytes");
                seed
            }
        })
    }
}

pub fn one_node_with_rng(rng: &mut Xoshiro256PlusPlus, dest: &mut String) {
    let mut unstructured_data = vec![0_u8; 1 << 12];
    loop {
        rng.fill_bytes(&mut unstructured_data);
        let mut u = Unstructured::new(&unstructured_data);
        if let Ok(node) = Node::arbitrary(&mut u) {
            node.write_into(dest);
            return;
        };
    }
}

pub fn one_node(seed: Seed) -> String {
    let mut src = String::new();
    one_node_with_rng(&mut seed.into_rng(), &mut src);
    src
}

pub fn min_size(seed: Seed, threshold: usize) -> String {
    assert!(threshold < isize::MAX as usize);

    let mut rng = seed.into_rng();
    let mut dest = String::new();
    while dest.len() < threshold {
        one_node_with_rng(&mut rng, &mut dest);
    }
    dest
}
