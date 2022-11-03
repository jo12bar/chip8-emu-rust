mod ram;
mod sys_font;

fn main() {
    let r = ram::Ram::new();
    println!("{r:x?}");
}
