// ============================================================
// 08 - STRUCTS (structs) - apna custom data type
// ============================================================
// Run karo:  cargo run --bin 08_structs
//
// Struct related data ko ek naam ke neeche group karta hai.

// -------- Classic struct (named fields) --------
struct User {
    username: String,
    email: String,
    age: u32,
    active: bool,
}

// -------- Tuple struct (bina field names ke) --------
struct Point(i32, i32);

// -------- Methods struct ke liye (impl block) --------
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    // Method - pehla parameter &self hota hai (khud struct ka reference)
    fn area(&self) -> u32 {
        self.width * self.height
    }

    // Doosre struct ke saath compare karne wala method
    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }

    // Associated function (self nahi leta) - aksar constructor ke tor pe.
    // Rectangle::square(10) se call hota hai.
    fn square(size: u32) -> Rectangle {
        Rectangle { width: size, height: size }
    }
}

fn main() {
    // Struct instance banana
    let mut user1 = User {
        username: String::from("ali_dev"),
        email: String::from("ali@example.com"),
        age: 25,
        active: true,
    };
    println!("User: {}, Email: {}", user1.username, user1.email);

    // Field change karna (instance mut hona chahiye)
    user1.age = 26;
    println!("Nayi umar: {}, Active: {}", user1.age, user1.active);

    // Tuple struct
    let origin = Point(0, 0);
    println!("Point: ({}, {})", origin.0, origin.1);

    // Methods use karna
    let rect1 = Rectangle { width: 30, height: 50 };
    let rect2 = Rectangle { width: 10, height: 40 };
    println!("rect1 ka area: {}", rect1.area());
    println!("Kya rect1 rect2 ko hold kar sakta hai? {}", rect1.can_hold(&rect2));

    // Associated function se square banana
    let sq = Rectangle::square(20);
    println!("Square ka area: {}", sq.area());
}
