// ============================================================
// 03 - FUNCTIONS (functions), PARAMETERS, RETURN VALUES
// ============================================================
// Run karo:  cargo run --bin 03_functions

// main() program ka entry point hota hai - yahin se shuru hota hai.
fn main() {
    // Simple function call
    greet();

    // Parameters ke saath function call
    greet_person("Sara");

    // Return value wala function
    let sum = add(5, 7);
    println!("5 + 7 = {}", sum);

    // Expression return (bina return keyword ke)
    let squared = square(4);
    println!("4 ka square = {}", squared);

    // Ek function jo doosre function ko call karta hai
    println!("Average = {}", average(10, 20));
}

// Function bina parameter, bina return value
fn greet() {
    println!("Salam! Rust me khush aamdeed.");
}

// Function with parameter - type likhna ZAROORI hai
// name: &str  ->  parameter ka naam aur uska type
fn greet_person(name: &str) {
    println!("Salam, {}!", name);
}

// Return value wala function.
// -> i32 ka matlab yeh function i32 return karega.
fn add(a: i32, b: i32) -> i32 {
    // NOTE: aakhri line me semicolon nahi hai.
    // Bina semicolon wali line "expression" hoti hai aur return ho jati hai.
    a + b
}

fn square(n: i32) -> i32 {
    n * n // yeh return ho jayega (koi semicolon nahi)
}

// `return` keyword bhi use kar sakte ho, khaas kar jaldi return karne ke liye.
fn average(a: i32, b: i32) -> i32 {
    return (a + b) / 2;
}
