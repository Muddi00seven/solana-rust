// ============================================================
// 02 - DATA TYPES (scalar aur compound types)
// ============================================================
// Run karo:  cargo run --bin 02_data_types

fn main() {
    // ---------------- SCALAR TYPES ----------------
    // Ek single value store karte hain.

    // 1) Integers (poore numbers)
    // Signed:   i8, i16, i32, i64, i128  (negative bhi ho sakte)
    // Unsigned: u8, u16, u32, u64, u128  (sirf 0 aur positive)
    let a: i32 = -42;   // default integer type i32 hai
    let b: u8 = 255;    // u8 sirf 0..=255 rakh sakta hai
    println!("i32: {}, u8: {}", a, b);

    // 2) Floating point (decimal numbers): f32, f64
    let pi: f64 = 3.14159; // default float f64 hai
    let e: f32 = 2.718;
    println!("f64: {}, f32: {}", pi, e);

    // 3) Boolean: true / false
    let is_rust_fun: bool = true;
    println!("Rust maza hai? {}", is_rust_fun);

    // 4) Character (char) - single character, single quotes me. Unicode support.
    let letter: char = 'R';
    let emoji: char = '🦀'; // Rust ka crab bhi ek char hai
    println!("char: {} {}", letter, emoji);

    // ---------------- COMPOUND TYPES ----------------
    // Multiple values ko ek me group karte hain.

    // 1) Tuple - different types ek saath. Fixed length.
    let person: (&str, i32, f64) = ("Ali", 25, 5.9);
    // Destructuring - tuple ko alag variables me tod do
    let (name, age, height) = person;
    println!("Naam: {}, Umar: {}, Height: {}", name, age, height);
    // Ya index se access karo (.0, .1, .2)
    println!("Pehla element: {}", person.0);

    // 2) Array - same type, fixed length. Stack pe store hota hai.
    let numbers: [i32; 5] = [1, 2, 3, 4, 5];
    println!("Array ka pehla: {}", numbers[0]);
    println!("Array ki length: {}", numbers.len());

    // Same value se array bhar do: [value; count]
    let zeros = [0; 3]; // [0, 0, 0]
    println!("Zeros: {:?}", zeros); // {:?} debug print ke liye
}
