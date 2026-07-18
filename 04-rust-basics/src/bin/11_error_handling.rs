// ============================================================
// 11 - ERROR HANDLING: Result, Option, panic!, ? operator
// ============================================================
// Run karo:  cargo run --bin 11_error_handling
//
// Rust me exceptions nahi hote. Errors do tarah handle hote hain:
//   - Recoverable errors  -> Result<T, E>
//   - Unrecoverable errors -> panic!

fn main() {
    // -------- Result<T, E> --------
    // Result ya to Ok(value) hota hai ya Err(error).
    match divide(10.0, 2.0) {
        Ok(result) => println!("10 / 2 = {}", result),
        Err(e) => println!("Error: {}", e),
    }

    match divide(10.0, 0.0) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {}", e), // yeh chalega
    }

    // -------- ? operator (error propagation) --------
    // ? ka matlab: agar Err hai to foran return kar do,
    // warna Ok ki value nikaal lo. Bohat clean tareeka.
    match parse_and_double("21") {
        Ok(n) => println!("Double: {}", n),
        Err(e) => println!("Parse error: {}", e),
    }
    match parse_and_double("abc") {
        Ok(n) => println!("Double: {}", n),
        Err(e) => println!("Parse error: {}", e), // yeh chalega
    }

    // -------- unwrap / expect --------
    // unwrap(): Ok/Some ki value deta hai, warna panic. (careful!)
    let safe: Result<i32, _> = "5".parse::<i32>();
    println!("Parsed: {}", safe.unwrap());

    // expect(): unwrap jaisa, lekin apna error message deta hai.
    let value: i32 = "100".parse().expect("Number valid nahi tha");
    println!("Value: {}", value);

    // -------- panic! (unrecoverable) --------
    // Yeh program crash kar deta hai. Sirf sach me kharaab halat me.
    // Comment hata kar dekho:
    // panic!("Sab kuch tabah ho gaya!");

    println!("Program theek chal raha hai.");
}

// Result return karne wala function.
// String error message ke tor pe use kiya.
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err(String::from("Zero se divide nahi kar sakte"))
    } else {
        Ok(a / b)
    }
}

// ? operator ka istemal: parse fail hua to Err khud return ho jayega.
fn parse_and_double(text: &str) -> Result<i32, std::num::ParseIntError> {
    let number: i32 = text.parse()?; // fail hua to yahin se Err return
    Ok(number * 2)
}
