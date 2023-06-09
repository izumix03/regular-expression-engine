mod engine;

fn main() {
    println!("Hello, world!");
}


#[cfg(test)]
mod tests {
    use std::mem::take;

    #[test]
    fn take_sample() {
        let mut n = Some(10);
        let v = take(&mut n);
        println!("n = {:?}, v = {:?}", n, v);
    }
}