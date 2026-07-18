// ============================================================
// 12 - GENERICS aur TRAITS
// ============================================================
// Run karo:  cargo run --bin 12_generics_traits
//
// Generics: ek hi code alag alag types ke liye (code reuse).
// Traits: shared behaviour define karte hain (interface jaisa).

// -------- Generic function --------
// <T> koi bhi type ho sakta hai. yahan T par PartialOrd chahiye
// taake > compare kar sakein.
fn largest<T: PartialOrd + Copy>(list: &[T]) -> T {
    let mut biggest = list[0];
    for &item in list {
        if item > biggest {
            biggest = item;
        }
    }
    biggest
}

// -------- Generic struct --------
// Point kisi bhi type T ka ho sakta hai (i32, f64, waghera).
struct Point<T> {
    x: T,
    y: T,
}

impl<T: std::fmt::Display> Point<T> {
    fn show(&self) {
        println!("Point: ({}, {})", self.x, self.y);
    }
}

// -------- Trait (shared behaviour) --------
// Trait ek "contract" hai: jo bhi ise implement kare, usay
// summarize() method dena hoga.
trait Summary {
    fn summarize(&self) -> String;

    // Default implementation bhi de sakte hain.
    fn preview(&self) -> String {
        String::from("(padho...)")
    }
}

struct Article {
    title: String,
    author: String,
}

struct Tweet {
    username: String,
    content: String,
}

// Article ke liye trait implement karo
impl Summary for Article {
    fn summarize(&self) -> String {
        format!("{} by {}", self.title, self.author)
    }
}

// Tweet ke liye trait implement karo (preview default use karega)
impl Summary for Tweet {
    fn summarize(&self) -> String {
        format!("@{}: {}", self.username, self.content)
    }
}

// Function jo koi bhi Summary type le sakta hai (trait bound)
fn notify(item: &impl Summary) {
    println!("Khabar! {}", item.summarize());
}

fn main() {
    // Generic function - alag types ke saath
    let numbers = vec![34, 50, 25, 100, 65];
    println!("Sab se bada number: {}", largest(&numbers));

    let chars = vec!['y', 'm', 'a', 'q'];
    println!("Sab se bada char: {}", largest(&chars));

    // Generic struct
    let int_point = Point { x: 5, y: 10 };
    let float_point = Point { x: 1.5, y: 4.2 };
    int_point.show();
    float_point.show();

    // Traits
    let article = Article {
        title: String::from("Rust seekho"),
        author: String::from("Ali"),
    };
    let tweet = Tweet {
        username: String::from("rustlang"),
        content: String::from("Ownership zabardast hai"),
    };

    notify(&article);
    notify(&tweet);
    println!("Tweet preview: {}", tweet.preview()); // default method
}
