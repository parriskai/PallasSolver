use pallas::lex::path;

fn main() {
    let input = include_str!("../../solution/src/lib.pa");
    println!("{:#?}", path(input));
}
