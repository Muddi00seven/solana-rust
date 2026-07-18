// ============================================================
// 13 - CLOSURES aur ITERATORS
// ============================================================
// Run karo:  cargo run --bin 13_closures_iterators
//
// Closure: anonymous function jo surrounding variables ko
//          capture kar sakta hai.
// Iterator: elements par ek ek kar ke process karne ka tareeka.

fn main() {
    // -------- Closures --------
    // |parameters| body  <- yeh closure ka syntax hai
    let add = |a: i32, b: i32| a + b;
    println!("Closure add: {}", add(3, 4));

    // Closure surrounding variable ko "capture" kar sakta hai
    let multiplier = 3;
    let times = |x: i32| x * multiplier; // multiplier capture hua
    println!("5 x {} = {}", multiplier, times(5));

    // Closure ko function me pass karna
    let result = apply(10, |x| x + 100);
    println!("apply result: {}", result);

    // -------- Iterators --------
    let numbers = vec![1, 2, 3, 4, 5];

    // map: har element ko transform karo
    // collect: wapas ek collection me jama karo
    let doubled: Vec<i32> = numbers.iter().map(|x| x * 2).collect();
    println!("Doubled: {:?}", doubled);

    // filter: sirf woh elements rakho jo condition pass karein
    let evens: Vec<&i32> = numbers.iter().filter(|x| *x % 2 == 0).collect();
    println!("Even numbers: {:?}", evens);

    // sum: sab ko jama karo
    let total: i32 = numbers.iter().sum();
    println!("Sum: {}", total);

    // Chaining: kai operations ek saath jodo
    let sum_of_doubled_evens: i32 = numbers
        .iter()
        .filter(|x| *x % 2 == 0) // pehle even chuno
        .map(|x| x * 2)          // phir double karo
        .sum();                  // phir jama karo
    println!("Even ko double kar ke sum: {}", sum_of_doubled_evens);

    // find: pehla element jo match kare
    let first_big = numbers.iter().find(|x| **x > 3);
    println!("Pehla 3 se bada: {:?}", first_big);

    // count / max / min
    println!("Count: {}", numbers.iter().count());
    println!("Max: {:?}", numbers.iter().max());
    println!("Min: {:?}", numbers.iter().min());
}

// Function jo ek closure ko parameter ke tor pe leta hai.
// F: Fn(i32) -> i32  ka matlab: koi bhi closure jo i32 le kar i32 de.
fn apply<F: Fn(i32) -> i32>(value: i32, func: F) -> i32 {
    func(value)
}
