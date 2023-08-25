fn main() {
    let mut test_vec = Vec::<u8>::new();
    loop {
        test_vec.push(41);
        let result = String::from_utf8(test_vec.clone());
        println!("{}", test_vec.len());
        if result.is_err() {
            break;
        }
    }
}
