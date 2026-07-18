// ============================================================
// 09 - ENUMS (enums), MATCH aur OPTION
// ============================================================
// Run karo:  cargo run --bin 09_enums
//
// Enum aisa type hai jiski value kai possible "variants" me se
// koi ek ho sakti hai.

// -------- Simple enum --------
enum Direction {
    North,
    South,
    East,
    West,
}

// -------- Enum with data (har variant apna data rakh sakta hai) --------
enum Message {
    Quit,                       // koi data nahi
    Move { x: i32, y: i32 },    // named fields (struct jaisa)
    Write(String),              // ek String
    ChangeColor(i32, i32, i32), // teen values
}

fn main() {
    // -------- Enum use karna + match --------
    let dir = Direction::West;
    // match har possible variant ko handle karta hai (exhaustive hona chahiye)
    match dir {
        Direction::North => println!("Upar ja rahe hain"),
        Direction::South => println!("Neeche ja rahe hain"),
        Direction::East => println!("Dayen ja rahe hain"),
        Direction::West => println!("Bayen ja rahe hain"),
    }

    // -------- Data wale enum ko match karna --------
    let msg = Message::Move { x: 10, y: 20 };
    process_message(msg);
    process_message(Message::Write(String::from("Salam")));
    process_message(Message::ChangeColor(255, 0, 0));
    process_message(Message::Quit);

    // -------- Option<T> - Rust ka null ka safe alternative --------
    // Option ya to Some(value) hota hai ya None. Isse "null pointer"
    // wale bugs compile time pe pakde jaate hain.
    let some_number: Option<i32> = Some(5);
    let no_number: Option<i32> = None;

    // match se Option handle karo
    match some_number {
        Some(n) => println!("Number mila: {}", n),
        None => println!("Kuch nahi mila"),
    }

    // `if let` - jab sirf ek case interesting ho (chhota shortcut)
    if let Some(n) = some_number {
        println!("if let se: {}", n);
    }

    // unwrap_or - None hone par default value de do
    println!("no_number ki value: {}", no_number.unwrap_or(0));
}

fn process_message(msg: Message) {
    match msg {
        Message::Quit => println!("Quit hua"),
        Message::Move { x, y } => println!("Move to ({}, {})", x, y),
        Message::Write(text) => println!("Text: {}", text),
        Message::ChangeColor(r, g, b) => println!("Color: {} {} {}", r, g, b),
    }
}
