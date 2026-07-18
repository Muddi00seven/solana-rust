// ============================================================
// 04 - CONTROL FLOW (if/else, loop, while, for)
// ============================================================
// Run karo:  cargo run --bin 04_control_flow

fn main() {
    // -------- if / else if / else --------
    let number = 7;
    if number % 2 == 0 {
        println!("{} even hai", number);
    } else {
        println!("{} odd hai", number);
    }

    // if ek EXPRESSION hai - value return kar sakta hai (ternary jaisa)
    let condition = true;
    let value = if condition { 5 } else { 10 };
    println!("value = {}", value);

    // -------- loop (infinite loop, break se rukta hai) --------
    // loop se value bhi return kar sakte ho break ke saath.
    let mut counter = 0;
    let result = loop {
        counter += 1;
        if counter == 10 {
            break counter * 2; // 20 return karega
        }
    };
    println!("loop se result: {}", result);

    // -------- while loop --------
    let mut n = 3;
    while n != 0 {
        println!("{}...", n);
        n -= 1;
    }
    println!("Chalo shuru!");

    // -------- for loop (sab se zyada use hone wala) --------
    // Array/collection ke har element pe ghoomo
    let arr = [10, 20, 30, 40, 50];
    for element in arr {
        println!("Value: {}", element);
    }

    // Range ke saath for loop (1..=5 => 1,2,3,4,5)
    // .. exclusive hai (aakhri number nahi), ..= inclusive hai
    for i in 1..=5 {
        println!("Number: {}", i);
    }

    // Ulta ghoomna .rev() ke saath
    for i in (1..4).rev() {
        println!("Reverse: {}", i);
    }
}
