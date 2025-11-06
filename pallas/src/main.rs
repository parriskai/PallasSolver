use pallas::parser::file;

fn main() {
    let input = include_str!("../../solution/lib.pa");
    println!("{:?}", file(input));
}
