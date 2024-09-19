use std::char::ToUppercase;

use util::{io::*, public::{StrBox, IO}};

fn main() {

    println!("Hello, world!");

    let data: Vec<Vec<StrBox>> = read_csv(&"land_council.csv").unwrap();

    // let landc: Hash<StrBox> = Hash:with_capacity(10);

    for x in data.iter() {

        let name = x[2].to_ascii_lowercase();

        if name.contains("land council") {

            println!("{:?}", x);

        }
    }

    
}
