use std::io;
use std::env;
use futures::join;
mod ciphers;

/// Main driver logic for user input
#[tokio::main]
async fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    loop {
        println!("Hello! What would you like to do today? Say 'help' to see cipher options, 'bruteforce' to attempt a bruteforce, or 'exit' to exit.");

        //Creates choice and var storage, mutable
        let mut user_choice = String::new();
        let mut user_vars = String::new();

        io::stdin().read_line(&mut user_choice).expect("Failed to read user input!");

        //Match input with contains conditionals (so user could type "caesar cipher please!" and it would still work fine)
        match user_choice.trim().to_lowercase() {
            opt if opt.contains("cae") || opt.contains("cea") => {
                println!("A caesar cipher is a common monoalphabetic substitution cipher that shifts letters by \
                a key called the shift value. Is this what you would like to do?");

                //read input
                io::stdin().read_line(&mut user_choice).expect("Failed to read user input!");


                let valid_yes_options = ["y"]; //creates array of valid options

                if valid_yes_options.iter().any(|&option| user_choice.trim().to_lowercase().contains(option)) { 
                    //Checks if either option is contained in the user's selection

                    println!("Please enter a comma separated list consisting of your message, the shift value, and whether you will be \
                    encrypting or decrypting the message. For example, \"secretmessage,8,enc\"");
    
                    io::stdin().read_line(&mut user_vars).expect("Failed to read user input!");
                    let args: Vec<&str> = user_vars.split(',').collect(); //split args by comma to get array of user inputted values
                    let valid_type_options = ["enc","dec"];
                    if let Some(val) = args.get(2) { //make sure there are 3 given values
                        if args[1].trim().to_lowercase().parse::<i32>().is_ok() && valid_type_options.iter().any(|&option| val.trim().to_lowercase().contains(option)) { 
                            let shift_key = args[1].trim().to_lowercase().parse::<i32>(); //Try to get shift key as integer
                            match shift_key {
                                Ok(shift_key) => {
                                    let result = ciphers::caesar_cipher(args[0],shift_key,val);
                                    let result_description = match val {
                                        x if x.contains("enc") => "ciphertext",
                                        x if x.contains("dec") => "plaintext",
                                        _=> "output",
                                    };
                                    println!("Resulting {} is: \t {}",result_description,result);
                                },
                                Err(_) => {
                                    println!("Shift key must be an integer, please try again.")
                                }
                            }
                        } else {
                            println!("Please enter a proper number of arguments");
                        }
                    }
                } else {
                    println!("Please try selecting a cipher again.");
                }
            }
            opt if opt.contains("vig") && !opt.contains("bru") => {
                println!("A vigenere cipher is a common polyalphabetic substitution cipher that shifts letters by \
                the values of a repeating key. Is this what you would like to do?");

                //read input
                io::stdin().read_line(&mut user_choice).expect("Failed to read user input!");


                let valid_yes_options = ["y"];
                if valid_yes_options.iter().any(|&option| user_choice.trim().to_lowercase().contains(option)) { 
                    println!("Please enter a comma separated list consisting of your message, the string key, and whether you will be \
                    encrypting or decrypting the message. For example, \"secretmessage,secretkey,enc\"");
    
                    io::stdin().read_line(&mut user_vars).expect("Failed to read user input!");
                    let args: Vec<&str> = user_vars.split(',').collect();
                    if let Some(val) = args.get(2) { //make sure there are 3 given values
                        let valid_type_options = ["enc","dec"];
                        if valid_type_options.iter().any(|&option| val.trim().to_lowercase().contains(option)) { 
                                let result = ciphers::vigenere_cipher(args[0],args[1],val);
                                let result_description = match val { //match input to get a nice output
                                    x if x.contains("enc") => "ciphertext",
                                    x if x.contains("dec") => "plaintext",
                                    _=> "output",
                                };
                                println!("Resulting {} is: \t {}",result_description,result);
                        } else {
                            println!("Couldn't locate 'enc' or 'dec' in reply!");
                        }
                    } else {
                        println!("Please enter a proper number of arguments");
                    }
                } else {
                    println!("Please try selecting a cipher again.");
                }
            }
            opt if opt.contains("atb") => {
                println!("An Atbash cipher is a common monoalphabetic substitution cipher that reverses the characters in a message. Is this what you would like to do?");

                //read input
                io::stdin().read_line(&mut user_choice).expect("Failed to read user input!");


                let valid_yes_options = ["y"];
                if valid_yes_options.iter().any(|&option| user_choice.trim().to_lowercase().contains(option)) { 
                    println!("Since the atbash cipher doesn't require a key and the encryption and decryption methods are the same, please enter only your secret message.");
    
                    io::stdin().read_line(&mut user_vars).expect("Failed to read user input!");
                    let result = ciphers::atbash_cipher(&user_vars);
                    println!("Resulting output is: \t {}", result);
                } else {
                    println!("Please try selecting a cipher again.");
                }
            }
            opt if opt.contains("rot") || opt.contains("13") => {
                println!("An ROT13 cipher is a simple substitution cipher that rotates each character by 13 spaces, as if choosing the other side of an alphabet wheel. Is this what you would like to do?");

                //read input
                io::stdin().read_line(&mut user_choice).expect("Failed to read user input!");


                let valid_yes_options = ["y"];
                if valid_yes_options.iter().any(|&option| user_choice.trim().to_lowercase().contains(option)) { 
                    println!("Since the ROT13 cipher doesn't require a key and the encryption and decryption methods are the same, please enter only your secret message.");
    
                    io::stdin().read_line(&mut user_vars).expect("Failed to read user input!");
                    let result = ciphers::rot13_cipher(&user_vars);
                    println!("Resulting output is: \t {}", result);
                } else {
                    println!("Please try selecting a cipher again.");
                }
            }
            opt if opt.contains("aff") => {
                println!("An affine cipher is a monoalphabetic substitution cipher that performs a mathematical operation, *a + b on a character, given the key [a,b]. a must be coprime to 26 (eg 3,5,7,9,11,15...). Is this what you would like to do?");

                //read input
                io::stdin().read_line(&mut user_choice).expect("Failed to read user input!");


                let valid_yes_options = ["y"];
                if valid_yes_options.iter().any(|&option| user_choice.trim().to_lowercase().contains(option)) { 
                    println!("Please enter a comma separated list consisting of your message, multiplicative key a, additive key b, and whether you will be \
                    encrypting or decrypting the message. For example, \"secretmessage,3,4,enc\"");
    
                    io::stdin().read_line(&mut user_vars).expect("Failed to read user input!");
                    let args: Vec<&str> = user_vars.split(',').collect();
                    if let Some(val) = args.get(3) { //make sure there are 4 given values
                        let valid_type_options = ["enc","dec"];
                        if valid_type_options.iter().any(|&option| val.trim().to_lowercase().contains(option)) { 
                                let mult_key_a = args[1].trim().to_lowercase().parse::<i32>().unwrap(); //Try to get key a as integer
                                let add_key_b = args[2].trim().to_lowercase().parse::<i32>().unwrap(); //Try to get key b as integer
                                let result = ciphers::affine_cipher(args[0],mult_key_a,add_key_b,val);
                                let result_description = match val { //match input to get a nice output
                                    x if x.contains("enc") => "ciphertext",
                                    x if x.contains("dec") => "plaintext",
                                    _=> "output",
                                };
                                println!("Resulting {} is: \t {}",result_description,result);
                        } else {
                            println!("Couldn't locate 'enc' or 'dec' in reply!");
                        }
                    } else {
                        println!("Please enter a proper number of arguments");
                    }
                } else {
                    println!("Please try selecting a cipher again.");
                }
            }
            opt if opt.contains("bac") => {
                println!("A baconian cipher is a monoalphabetic substitution cipher that encodes the message in a sort of binary using 'a's and 'b's, fonts or cases, or in this case, randomized digits where digits 6 and below are 0's and 7 and above are 1's. Each character is stored in 5 bits representing the ASCII. Is this what you would like to do?");

                //read input
                io::stdin().read_line(&mut user_choice).expect("Failed to read user input!");


                let valid_yes_options = ["y"];
                if valid_yes_options.iter().any(|&option| user_choice.trim().to_lowercase().contains(option)) { 
                    println!("Please enter a comma separated list consisting of your message and whether you will be \
                    encrypting or decrypting the message. For example, \"secretmessage,enc\"");
    
                    io::stdin().read_line(&mut user_vars).expect("Failed to read user input!");
                    let args: Vec<&str> = user_vars.split(',').collect();
                    if let Some(val) = args.get(1) { //make sure there are 2 given values
                        let valid_type_options = ["enc","dec"];
                        if valid_type_options.iter().any(|&option| val.trim().to_lowercase().contains(option)) { 
                                let result = ciphers::baconian_cipher(args[0],val);
                                let result_description = match val { //match input to get a nice output
                                    x if x.contains("enc") => "ciphertext",
                                    x if x.contains("dec") => "plaintext",
                                    _=> "output",
                                };
                                println!("Resulting {} is: \t {}",result_description,result);
                        } else {
                            println!("Couldn't locate 'enc' or 'dec' in reply!");
                        }
                    } else {
                        println!("Please enter a proper number of arguments");
                    }
                } else {
                    println!("Please try selecting a cipher again.");
                }
            }
            opt if opt.contains("rai") => {
                println!("A Railfence cipher is a transposition cipher that shuffles each character according to a number of rails that act as the key. Is this what you would like to do?");

                //read input
                io::stdin().read_line(&mut user_choice).expect("Failed to read user input!");


                let valid_yes_options = ["y"];
                if valid_yes_options.iter().any(|&option| user_choice.trim().to_lowercase().contains(option)) { 
                    println!("Please enter a comma separated list consisting of your message, the number of rails you'd like to use (this should be less than the number of characters in the message), and whether you will be \
                    encrypting or decrypting the message. For example, \"secretmessage,3,enc\"");
    
                    io::stdin().read_line(&mut user_vars).expect("Failed to read user input!");
                    let args: Vec<&str> = user_vars.split(',').collect();
                    if let Some(val) = args.get(2) { //make sure there are 2 given values
                        let valid_type_options = ["enc","dec"];
                        if valid_type_options.iter().any(|&option| val.trim().to_lowercase().contains(option)) { 

                            let rail_int = args[1].trim().to_lowercase().parse::<i32>(); //Try to get shift key as integer
                            match rail_int {
                                Ok(rail_int) => {                                
                                    let result = ciphers::railfence_cipher(args[0],rail_int,val);
                                    let result_description = match val {
                                        x if x.contains("enc") => "ciphertext",
                                        x if x.contains("dec") => "plaintext",
                                        _=> "output",
                                    };
                                    println!("Resulting {} is (remove the single quotes!): \t {}",result_description,result);
                                },
                                Err(_) => {
                                    println!("Shift key must be an integer, please try again.")
                                }
                            }
                        } else {
                            println!("Couldn't locate 'enc' or 'dec' in reply!");
                        }
                    } else {
                        println!("Please enter a proper number of arguments");
                    }
                } else {
                    println!("Please try selecting a cipher again.");
                }
            }
            opt if opt.contains("bru") && !opt.contains("vig") => {
                println!("This will attempt a bruteforce on a string encoded using one of the available cipher types. Is this what you would like to do?");

                //read input
                io::stdin().read_line(&mut user_choice).expect("Failed to read user input!");

                let valid_yes_options = ["y"];
                if valid_yes_options.iter().any(|&option| user_choice.trim().to_lowercase().contains(option)) { 
                    println!("Please enter a comma separated list consisting of the encrypted text, followed by the encryption type if you know it. This will not work for a vigenere cipher. For example, \"encryptedmessage,railcipher\" or simply \"encryptedmessage\"");
    
                    io::stdin().read_line(&mut user_vars).expect("Failed to read user input!");
                    let args: Vec<&str> = user_vars.split(',').collect();
                    if let Some(val) = args.get(1) { //check if there are 2 values
                        ciphers::bruteforce(args[0],val);
                    } else {
                        ciphers::bruteforce(args[0], "unknown");
                    }
                } else {
                    println!("Please try selecting a cipher again.");
                }
            }
            opt if opt.contains("sco") => {
                println!("This will score a string. Enter a string to score.");
                let mut user_choice = String::new();
                io::stdin().read_line(&mut user_choice).expect("Failed to read user input!");

                let mut word_list: Vec<String> = vec![];
                if let Ok(lines) = ciphers::read_lines("src/data/1000_most_common.txt") {
                    // Consumes the iterator, returns an (Optional) String
                    for line in lines.flatten() {
                        word_list.push(line);
                    }
                } else {println!("Directory not found!")}
                println!("The score for this string is: {}", ciphers::score_string(&user_choice, &word_list))
            }
            opt if opt.contains("bru") && opt.contains("vig") => {
                println!("This will attempt a bruteforce on a string encoded with vigenere. Note that vigenere will take a long time, and may not be possible given a secure enough key. Is this what you would like to do?");

                //read input
                io::stdin().read_line(&mut user_choice).expect("Failed to read user input!");

                let valid_yes_options = ["y"];
                if valid_yes_options.iter().any(|&option| user_choice.trim().to_lowercase().contains(option)) { 
                    println!("Please enter the encrypted text and a security level from 0 to 100 separated by a comma. Higher security levels will check more passwords but will take much longer. 5 is a good start for weak passwords. For example, \"encryptedmessage,5\"");
                    let mut bfl = (5.0 / 100.0 * 14344392.0) as i32; //14344392 is the number of passwords in the bruteforce list
                    io::stdin().read_line(&mut user_vars).expect("Failed to read user input!");
                    let args: Vec<&str> = user_vars.split(',').collect();
                    if args.get(1).is_some() && args[1].trim().to_lowercase().parse::<i32>().is_ok() && args[1] != "0" { //check if 2 args are given, if not we will default to 5.
                        bfl = (args[1].trim().to_lowercase().parse::<i32>().unwrap() as f64 / 100.0 * 14344392.0) as i32; //14344392 is the number of passwords in the bruteforce list
                        let out = ciphers::bruteforce_vigenere(&args[0],bfl);
                        let _ = join!(out);
                        
                    } else {
                        let out = ciphers::bruteforce_vigenere(&args[0],bfl);
                        let _ = join!(out);
                    }
                } else {
                    println!("Please try selecting a cipher again.");
                }
            }

            opt if opt.contains("pol") => {
                println!("A Polybius cipher is a monoalphabetic cipher that shifts values by one row according to a 5x5 alphabetic table. Is this what you would like to do?");

                //read input
                io::stdin().read_line(&mut user_choice).expect("Failed to read user input!");


                let valid_yes_options = ["y"];
                if valid_yes_options.iter().any(|&option| user_choice.trim().to_lowercase().contains(option)) { 
                    println!("Please enter a comma separated list consisting of your message and whether you will be \
                    encrypting or decrypting the message. For example, \"secretmessage,enc\"");
    
                    io::stdin().read_line(&mut user_vars).expect("Failed to read user input!");
                    let args: Vec<&str> = user_vars.split(',').collect();
                    if let Some(val) = args.get(1) { //make sure there are 2 given values
                        let valid_type_options = ["enc","dec"];
                        if valid_type_options.iter().any(|&option| val.trim().to_lowercase().contains(option)) { 
                                let result = ciphers::polybius_cipher(args[0],val);
                                let result_description = match val { //match input to get a nice output
                                    x if x.contains("enc") => "ciphertext",
                                    x if x.contains("dec") => "plaintext",
                                    _=> "output",
                                };
                                println!("Resulting {} is: \t {}",result_description,result);
                        } else {
                            println!("Couldn't locate 'enc' or 'dec' in reply!");
                        }
                    } else {
                        println!("Please enter a proper number of arguments");
                    }
                } else {
                    println!("Please try selecting a cipher again.");
                }
            }

            opt if opt.contains("sim") => {
                println!("A simple subsitution cipher is a common monoalphabetic substitution cipher that shifts letters by random values seeded by a given key password. Is this what you would like to do?");

                //read input
                io::stdin().read_line(&mut user_choice).expect("Failed to read user input!");


                let valid_yes_options = ["y"]; //creates array of valid options

                if valid_yes_options.iter().any(|&option| user_choice.trim().to_lowercase().contains(option)) { 
                    //Checks if either option is contained in the user's selection

                    println!("Please enter a comma separated list consisting of your message, the shift value, and whether you will be \
                    encrypting or decrypting the message. For example, \"secretmessage,8,enc\"");
    
                    io::stdin().read_line(&mut user_vars).expect("Failed to read user input!");
                    let args: Vec<&str> = user_vars.split(',').collect(); //split args by comma to get array of user inputted values
                    let valid_type_options = ["enc","dec"];
                    if let Some(val) = args.get(2) { //make sure there are 3 given values
                        if valid_type_options.iter().any(|&option| val.trim().to_lowercase().contains(option)) { 
                                    let result = ciphers::simplesub_cipher(args[0],args[1],val);
                                    let result_description = match val {
                                        x if x.contains("enc") => "ciphertext",
                                        x if x.contains("dec") => "plaintext",
                                        _=> "output",
                                    };
                                    println!("Resulting {} is: \t {}",result_description,result);
                        } else {
                            println!("Please enter a proper number of arguments");
                        }
                    }
                } else {
                    println!("Please try selecting a cipher again.");
                }
            }

            opt if opt.contains("help") => {
                println!("Enter a valid cipher option. Valid options include the following:\n\n
caesar cipher: Shift characters by integer shift key,\n
vigenere cipher: Shift characters by repeating string key,\n
atbash cipher: Reverse characters (a => z, b => y, ...),\n
Affine cipher: Performs *a+b on chars to encrypt, /a-b to decrypt,\n
Baconian cipher: Encodes text as an integer stream which represents binary,\n
Polybius cipher: Encodes text by substituting values according to a table,\n
Simple Substitution cipher: Encodes text by substituting values according to a seeded shuffle,\n
Railfence cipher: Shuffles the order of the characters using a zig-zag pattern along a # of rails, which act as the key,\n
ROT13 cipher: Shift characters by 13 places,\n

You can also enter bruteforce to bruteforce any cipher other than vigenere and simple-substitution, or bruteforce vigenere to bruteforce a vigenere cipher.\n
Note: You don't need to enter the full name, you only have to enter enough of the name to register as uniquely one cipher (ie, cae and vig both will work)\n");
            }
            opt if opt.contains("exit") => {
                println!("Exiting program!");
                break;
            }
            _ => {
                println!("No cipher was detected! Please try again.");
            }
        }
    }
}
