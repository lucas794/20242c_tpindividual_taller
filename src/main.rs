

fn main() {
    // lets read the args
    let args: Vec<String> = std::env::args().collect();

    if !valid_number_of_args(&args.len()) {
        return; // we stop the program, invalid number of arguments
    }

    // Now, we have the arguments...
    let file = &args[1];
    let consult = &args[2].trim().to_string();

    println!("File: {}", file);
    println!("Consult: [{}]", consult);
}

fn valid_number_of_args(args: &usize) -> bool {
    if *args <= 2 {
        println!("Invalid number of arguments!");
        return false;
    }
    true
}