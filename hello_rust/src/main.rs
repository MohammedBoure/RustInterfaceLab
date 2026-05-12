fn main() {
    let numbers = [3, 7, 11, 19];
    let total: i32 = numbers.iter().sum();

    println!("Rust is ready.");
    println!("Numbers: {:?}", numbers);
    println!("Total: {total}");
}
