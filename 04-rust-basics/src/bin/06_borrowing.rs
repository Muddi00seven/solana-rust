// ============================================================
// 06 - REFERENCES & BORROWING (references aur borrowing)
// ============================================================
// Run karo:  cargo run --bin 06_borrowing
//
// Borrowing ka matlab: value ki ownership liye bina uska
// reference (&) le lo. Isse value move nahi hoti.

fn main() {
    // -------- Immutable reference (&) --------
    let s1 = String::from("hello");
    // &s1 se hum s1 ko "borrow" kar rahe hain (ownership nahi li)
    let len = calculate_length(&s1);
    // s1 abhi bhi valid hai kyunki humne sirf reference diya tha
    println!("'{}' ki length {} hai", s1, len);

    // -------- Mutable reference (&mut) --------
    // Value change karni ho reference se to &mut chahiye.
    let mut s2 = String::from("hello");
    change(&mut s2);
    println!("Change ke baad: {}", s2);

    // -------- Borrowing ke rules --------
    // Rule 1: Ek waqt me ya to EK mutable reference,
    //         ya JITNE chaho immutable references. Dono ek saath nahi.
    let mut s3 = String::from("data");
    {
        let r1 = &s3; // immutable
        let r2 = &s3; // immutable - multiple OK
        println!("r1={}, r2={}", r1, r2);
    } // r1, r2 yahan khatam

    let r3 = &mut s3; // ab mutable reference OK hai
    r3.push_str("!!!");
    println!("r3 = {}", r3);

    // Yeh rule "data races" ko compile time pe rok deta hai.
}

// &String ka matlab: yeh sirf reference le raha hai, ownership nahi.
fn calculate_length(s: &String) -> usize {
    s.len()
} // s scope se bahar gaya lekin ownership nahi thi, isliye kuch drop nahi hota

// &mut String -> mutable reference, value ko andar se badal sakte hain.
fn change(some_string: &mut String) {
    some_string.push_str(", world");
}
