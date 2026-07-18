// ============================================================
// 05 - OWNERSHIP (Rust ka sab se khaas concept)
// ============================================================
// Run karo:  cargo run --bin 05_ownership
//
// Ownership Rust ki memory safety ka core hai - bina garbage
// collector ke memory manage hoti hai. 3 rules:
//   1) Har value ka ek "owner" hota hai.
//   2) Ek waqt me sirf ek owner ho sakta hai.
//   3) Jab owner scope se bahar jaye to value drop (free) ho jati hai.

fn main() {
    // -------- Stack data (Copy) --------
    // Simple types (integers, bool, char, float) Stack pe hote hain
    // aur inka "Copy" ho jata hai. Purana variable valid rehta hai.
    let x = 5;
    let y = x; // yahan x ki COPY bani, dono valid hain
    println!("x = {}, y = {}", x, y);

    // -------- Heap data (Move) --------
    // String heap pe store hoti hai. Jab assign karo to ownership
    // MOVE ho jati hai - purana variable invalid ho jata hai.
    let s1 = String::from("hello");
    let s2 = s1; // ownership s1 se s2 me MOVE ho gayi
    // println!("{}", s1); // <-- ERROR! s1 ab valid nahi (moved)
    println!("s2 = {}", s2);

    // -------- Clone (jaan boojh kar copy) --------
    // Agar sach me deep copy chahiye to .clone() use karo.
    let s3 = String::from("world");
    let s4 = s3.clone(); // dono ki apni apni copy, dono valid
    println!("s3 = {}, s4 = {}", s3, s4);

    // -------- Ownership aur functions --------
    let s = String::from("Rust");
    takes_ownership(s); // s ki ownership function me move ho gayi
    // println!("{}", s); // <-- ERROR! s move ho chuki hai

    let num = 10;
    makes_copy(num); // num ki copy gayi, num abhi bhi valid hai
    println!("num abhi bhi valid: {}", num);

    // -------- Function se ownership wapas lena --------
    let s5 = gives_ownership(); // function ne value return kar ke ownership di
    println!("Function se mili: {}", s5);
}

// Yeh function String ki ownership le leta hai.
fn takes_ownership(some_string: String) {
    println!("Function ke andar: {}", some_string);
} // yahan some_string drop ho jati hai (memory free)

// Yeh integer ki copy leta hai (Copy type).
fn makes_copy(some_int: i32) {
    println!("Copy mili: {}", some_int);
}

// Yeh function ek String bana kar uski ownership return karta hai.
fn gives_ownership() -> String {
    String::from("yours")
}
