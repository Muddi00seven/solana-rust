// ============================================================
// 01 - VARIABLES (variables), MUTABILITY, CONSTANTS, SHADOWING
// ============================================================
// Run karo:  cargo run --bin 01_variables

fn main() {
    // -------- Immutable variable (default) --------
    // Rust me variable by default IMMUTABLE hota hai.
    // Matlab ek dafa value set ki to badal nahi sakte.
    let x = 5;
    println!("x ki value: {}", x);
    // x = 6; // <-- ERROR! immutable ko dobara assign nahi kar sakte

    // -------- Mutable variable --------
    // Agar value change karni ho to `mut` keyword lagao.
    let mut y = 10;
    println!("y pehle: {}", y);
    y = 20; // ab yeh allowed hai kyunki y mutable hai
    println!("y ab: {}", y);

    // -------- Constant --------
    // const hamesha immutable hota hai, type likhna zaroori hai,
    // aur yeh compile-time pe fix ho jata hai. Naam UPPER_CASE me.
    const MAX_POINTS: u32 = 100_000; // _ sirf readability ke liye
    println!("Max points: {}", MAX_POINTS);

    // -------- Shadowing --------
    // Same naam se naya variable bana sakte ho `let` dobara use kar ke.
    // Yeh purani value ko "shadow" (dhak) leta hai. Type bhi change ho sakta hai.
    let z = 5;
    let z = z + 1; // ab z = 6
    let z = z * 2; // ab z = 12
    println!("Shadowed z: {}", z);

    // Shadowing me type badal sakte ho (mut me nahi):
    let spaces = "   ";        // yeh string hai
    let spaces = spaces.len(); // ab yeh number ban gaya (usize)
    println!("Spaces count: {}", spaces);
}
