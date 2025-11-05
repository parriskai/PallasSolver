use pallas::parser::define;

fn main() {
    let input = include_str!("../../basic.pa");
    println!("{:#?}", define(input))
}
