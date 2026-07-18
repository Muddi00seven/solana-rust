// ============================================================
// 07 - SLICES (slices) - collection ke ek hissey ka reference
// ============================================================
// Run karo:  cargo run --bin 07_slices
//
// Slice ownership liye bina collection ke ek part ko refer karta hai.

fn main() {
    // -------- String slice (&str) --------
    let s = String::from("hello world");

    // [start..end]  -> start included, end excluded
    let hello = &s[0..5];  // index 0 se 4 tak -> "hello"
    let world = &s[6..11]; // index 6 se 10 tak -> "world"
    println!("{} | {}", hello, world);

    // Shortcuts:
    let from_start = &s[..5];  // shuru se 5 tak (0..5 jaisa)
    let to_end = &s[6..];      // 6 se aakhir tak
    let whole = &s[..];        // poori string
    println!("{} | {} | {}", from_start, to_end, whole);

    // Practical example: pehla word nikalna
    let first = first_word(&s);
    println!("Pehla word: {}", first);

    // -------- Array slice --------
    let arr = [1, 2, 3, 4, 5];
    let slice = &arr[1..4]; // [2, 3, 4]
    println!("Array slice: {:?}", slice);
    println!("Slice ki length: {}", slice.len());
}

// &str return type use karte hain taake String aur &str dono chalein.
fn first_word(s: &str) -> &str {
    let bytes = s.as_bytes(); // string ko bytes me convert
    // .iter().enumerate() se (index, value) milta hai
    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            // pehla space mila -> uss se pehle wala hissa return karo
            return &s[0..i];
        }
    }
    // koi space nahi mila -> poori string ek hi word hai
    &s[..]
}
