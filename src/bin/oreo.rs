fn main() {
    let zippie = (1..10).zip(1..20);
    for i in zippie {
        print!("{i:?}");
    }
}