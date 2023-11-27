pub mod blocktype;
pub mod duration;
pub mod time;
pub mod timeblock;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {
    let blocks = blocktype::BlockType::load().unwrap();
    println!("{:#?}", blocks);
}
