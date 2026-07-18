// ============================================================
// 10 - COLLECTIONS: Vec, String, HashMap
// ============================================================
// Run karo:  cargo run --bin 10_collections
//
// Yeh collections heap pe store hoti hain aur grow/shrink ho sakti hain.

use std::collections::HashMap; // HashMap import karna zaroori hai

fn main() {
    // ---------------- VECTOR (Vec<T>) ----------------
    // Ek hi type ki growable list.
    let mut numbers: Vec<i32> = Vec::new();
    numbers.push(1); // add karo
    numbers.push(2);
    numbers.push(3);
    println!("Vector: {:?}", numbers);

    // Shortcut: vec! macro se banao
    let mut fruits = vec!["apple", "banana"];
    fruits.push("mango");
    println!("Fruits: {:?}", fruits);

    // Element access - index se (panic ho sakta hai agar range se bahar)
    println!("Pehla number: {}", numbers[0]);
    // .get() safe hai - Option deta hai
    match numbers.get(10) {
        Some(n) => println!("Value: {}", n),
        None => println!("Index 10 pe kuch nahi"),
    }

    // Vector par loop
    for n in &numbers {
        println!("Number: {}", n);
    }

    // ---------------- STRING ----------------
    let mut s = String::from("Salam");
    s.push_str(", duniya"); // string add karo
    s.push('!');            // ek char add karo
    println!("String: {}", s);

    // Concatenation with format! (aasan tareeka)
    let a = String::from("Hello");
    let b = String::from("Rust");
    let combined = format!("{} {}", a, b);
    println!("Combined: {}", combined);

    // ---------------- HASHMAP ----------------
    // Key -> Value pairs (jaisa dictionary/object).
    let mut scores: HashMap<String, i32> = HashMap::new();
    scores.insert(String::from("Ali"), 90);
    scores.insert(String::from("Sara"), 85);

    // Value get karna (Option deta hai)
    match scores.get("Ali") {
        Some(score) => println!("Ali ka score: {}", score),
        None => println!("Ali nahi mila"),
    }

    // HashMap par loop (order fixed nahi hota)
    for (name, score) in &scores {
        println!("{}: {}", name, score);
    }

    // entry() - agar key nahi hai to hi insert karo
    scores.entry(String::from("Bilal")).or_insert(70);
    println!("Final scores: {:?}", scores);
}
