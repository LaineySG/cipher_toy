use std::io;
mod ciphers;

fn main() {
    loop {
        println!("Hello! What would you like to do today? To see options, say 'help', or to exit, say 'exit'.");

        //Creates choice and var storage, mutable
        let mut user_choice = String::new();
        let mut user_vars = String::new();

        io::stdin().read_line(&mut user_choice).expect("Failed to read user input!");

        //check for input, if there 
        let valid_ceasar_options = ["cea", "cae", "1"];
        if valid_ceasar_options.iter().any(|&option| user_choice.trim().to_lowercase().contains(option)) { //if caesar is chosen
            println!("A caesar cipher is a common substitution cipher that shifts letters by \
            a key called the shift value. Is this what you would like to do?");

            //read input
            io::stdin().read_line(&mut user_choice).expect("Failed to read user input!");


            let valid_yes_options = ["y","1"];
            if valid_yes_options.iter().any(|&option| user_choice.trim().to_lowercase().contains(option)) { 
                println!("Please enter a comma separated list consisting of your message, the shift value, and whether you will be \
                encrypting or decrypting the message. For example, \"secretmessage,8,enc\"");
 
                io::stdin().read_line(&mut user_vars).expect("Failed to read user input!");
                let args: Vec<&str> = user_vars.split(',').collect();
                let valid_type_options = ["enc","dec"];
                if args[1].trim().to_lowercase().parse::<i32>().is_ok() && valid_type_options.iter().any(|&option| args[2].trim().to_lowercase().contains(option)) { 
                    let shift_key = args[1].trim().to_lowercase().parse::<i32>();
                    match shift_key {
                        Ok(shift_key) => {
                            let ciphertext = ciphers::caesar_cipher(args[0],shift_key,args[2]);
                            println!("Your ciphertext is: \t {}",ciphertext);
                        },
                        Err(_) => {
                            println!("Shift key must be an integer, please try again.")
                        }
                    }
                } else {
                    println!("There was an error with this input!")
                }
            } else {
                println!("Please try selecting a cipher again.");
            }
        } else {
            println!("Your response didn't seem to have any cipher name in it. Please try again.");
        }
        //Check exit condition
        if user_choice.trim().to_lowercase().contains("exit") {
            println!("Exiting program!");
            break;
        }
        //Check help condition
        if user_choice.trim().to_lowercase().contains("help") {
            println!("Enter a valid cipher option. Valid options include the following: 
            caesar cipher,");
        }
    }
}
