use std::collections::HashMap;
use ascii::AsciiStr;
use core::cmp::Ordering;
use modinverse::modinverse;
use rand::Rng;
const LOWERCASE_ASCII_OFFSET: i32 = 97;
const UPPERCASE_ASCII_OFFSET: i32 = 65;
const INTEGER_ASCII_OFFSET: i32 = 48; //48 is 0, 57 is 9
const LETTER_LIKELIHOOD_DICT: [f64;26] = [
    0.08167, 0.01492, 0.02782, 0.04253, 0.12702, 0.02228, 0.02015, 0.06094, 
    0.06966, 0.00153, 0.00772, 0.04025, 0.02406, 0.06749, 0.07507, 0.01929, 
    0.00095, 0.05987, 0.06327, 0.09056, 0.02758, 0.00978, 0.02360, 0.00150, 
    0.01974, 0.00074];


/// Shifts character while keeping it in a safe range of characters (stopping newline and other weird ascii chars as well as potential overflow)
pub fn shift_char(c: char, shift: i32) -> char {
    if (c as u8) < 48 || (c as u8) > 126 { //if it's a weird character don't shift it
        return c
    }
    let shifted_value = c as u8 as i32 + shift; //Shift the value. rem_euclid takes modulus basically, but for signed numbers. This keeps it in range of 48 to 126
    let wrapped_value = (shifted_value - 48).rem_euclid(79) + 48;
    wrapped_value as u8 as char
}
 /// Caesar cipher shifts the values of each character in the message by a set amount, the shift key. To decrypt, it simply reverses this (shifting backwards).
pub fn caesar_cipher(message: &str, shift: i32, enc_type: &str) -> String {
    let mut result = String::new();
    let shift = if enc_type.contains("dec") { -shift } else { shift }; //If we're decrypting, shift should be backwards.
    for c in message.chars() { //For each character in the message to decrypt, we shift that char and push it to result
        result.push(shift_char(c, shift));
    }
    result //Then return result
}

/// Vigenere cipher shifts the values of each character in the message by the value of a character in a repeating key. 
/// To decrypt, it simply reverses this (shifting backwards).
pub fn vigenere_cipher(message: &str, key: &str, enc_type: &str) -> String {
    let mut result = String::new();
    let mut key_cursor:usize = 0;

    for (_idx, current_char) in message.chars().enumerate() { //returns index and char for each char in message.
        if key_cursor >= key.chars().count() { //If cursor is out of bounds (at the end of the key), reset it
            key_cursor = 0;
        }

        //Converts the key to ascii, then slices it into an array of ascii characters (so it's indexed properly)
        let indexed_key = AsciiStr::from_ascii(key).unwrap(); 
        let key_ascii_arr = indexed_key.as_slice();
        

        let mut shift = 0;

        //Grab the value of the key as an integer, subtract the base ascii offset for lowercase characters and save it as the shift value. 
        //Decrypt is same but shift becomes negative
        if enc_type.contains("enc") {
            shift = (key_ascii_arr[key_cursor] as i32) - LOWERCASE_ASCII_OFFSET;
        } else if enc_type.contains("dec") {
            shift = -((key_ascii_arr[key_cursor] as i32) - LOWERCASE_ASCII_OFFSET);
        }

        key_cursor += 1;
        println!("Shifted to char: {}, {}", &shift_char(current_char, shift).to_string(), shift_char(current_char, shift) as u8);
        result += &shift_char(current_char, shift).to_string(); //Finally, add the shifted char as a string, to result.
    }
    result
}

/// The atbash cipher takes a message and reverses all characters in the string.
pub fn atbash_cipher(message: &str) -> String {
    let mut result = String::new();
    for c in message.chars() { //For each character in the message to decrypt, we reverse that char and push it to result
        let out = match c {
            x if x.is_uppercase() => { //if uppercase, take char as int, subtract the uppercase ascii offset, then take 25 and subtract the result from it to get the new ascii digit, and finally add the offset back and return a char.
                (25 - (((c as u8) as i32) - UPPERCASE_ASCII_OFFSET) + UPPERCASE_ASCII_OFFSET) as u8 as char
            },
            x if x.is_lowercase() => {
                (25 - (((c as u8) as i32) - LOWERCASE_ASCII_OFFSET) + LOWERCASE_ASCII_OFFSET) as u8 as char
            },
            x if x.to_string().parse::<i32>().is_ok() => { //if it parses as integer, we do same as above but w/ integer conversions
                (9 - (((c as u8) as i32) - INTEGER_ASCII_OFFSET) + INTEGER_ASCII_OFFSET) as u8 as char
            },
            _ => {
                c
            }
        };
        result.push_str(&out.to_string());
    }
    result
}

/// The rot13 cipher takes a message and rotates all alpha chars by 13 as if on a wheel.
pub fn rot13_cipher(message: &str) -> String {
    let mut result = String::new();
    const ROT13_SHIFT: i32 = 13;
    for c in message.chars() { //For each character in the message to decrypt, we reverse that char and push it to result
        let out = match c {
            x if x.is_uppercase() => { //if uppercase, take char as int, subtract the uppercase ascii offset, then take 25 and subtract the result from it to get the new ascii digit, and finally add the offset back and return a char.
                //(25 - (((c as u8) as i32) - UPPERCASE_ASCII_OFFSET) + UPPERCASE_ASCII_OFFSET) as u8 as char
                ((((c as u8 as i32) - UPPERCASE_ASCII_OFFSET + ROT13_SHIFT) % 26) + UPPERCASE_ASCII_OFFSET) as u8 as char
            },
            x if x.is_lowercase() => {
                ((((c as u8 as i32) - LOWERCASE_ASCII_OFFSET + ROT13_SHIFT) % 26) + LOWERCASE_ASCII_OFFSET) as u8 as char
            },
            _ => { //else return that char
                c
            }
        };
        result.push_str(&out.to_string());
    }
    result
}

/// The affine cipher takes a message and performs the modification: *a + b on each character. To decrypt, it performs /a - b.
pub fn affine_cipher(message: &str, a: i32, b: i32, enc_type: &str) -> String {
    let mut result = String::new();
    for c in message.chars() {
        let out = match c {
            c if c.is_uppercase() => {
                if enc_type.contains("enc") {
                    (((((c as u8 as i32) - UPPERCASE_ASCII_OFFSET) * a + b) % 26) + UPPERCASE_ASCII_OFFSET) as u8 as char
                } else {
                    let inverse_modulo = modinverse(a, 26).unwrap(); //calculates the inverse modulo
                    let char_dec = (c as u8 as i32) - UPPERCASE_ASCII_OFFSET; //character value from 0 to 26   

                    (((inverse_modulo * (char_dec - b + 26)) % 26 ) + UPPERCASE_ASCII_OFFSET) as u8 as char 
                    //undoes adding b, add 26 to ensure it's positive, multiply by inverse modulo (undoes modulo), takes mod 26 so the result will be between 0 and 25,
                    //and finally adds the offset and converts back to char
                }
            },
            c if c.is_lowercase() => {
                if enc_type.contains("enc") {
                    (((((c as u8 as i32) - LOWERCASE_ASCII_OFFSET) * a + b) % 26) + LOWERCASE_ASCII_OFFSET) as u8 as char
                } else {
                    let inverse_modulo = modinverse(a, 26).unwrap(); 
                    let char_dec = (c as u8 as i32) - LOWERCASE_ASCII_OFFSET; //character value from 0 to 26                   
                    (((inverse_modulo * (char_dec - b + 26)) % 26 ) + LOWERCASE_ASCII_OFFSET) as u8 as char
                }
            },
            _ => {
                c
            },
        };
        result.push_str(&out.to_string());
    }
    result
}

/// A baconian cipher converts characters to binary, then that binary is changed such that 0's are a's, and 1's are b's. It can also use different
/// typefaces such as CAPS for 1's and lowercase for 0's. This particular method creates a random string of numbers where high numbers signify 1's and low
/// numbers signify 0's. This helps to obfuscate the data.
pub fn baconian_cipher(message:&str, enc_type: &str) -> String {
    //converts ascii chars to digits, then 5bit binary. This can then be converted to strings of random numbers 
    //where 0,1,2,3,4,5,6 are binary 0's and 7,8,9 are binary 1's. 
    let mut result = String::new();
    if enc_type.contains("enc") {
        for c in message.chars() {
            match c {
                c if c.is_whitespace() => { //return whitespace
                    result.push_str(&c.to_string());
                },
                c if c.is_uppercase() => {
                    let _char_dec = c as u8 as i32 - UPPERCASE_ASCII_OFFSET; //Get the decimal char value
                    let char_bin = format!("{_char_dec:05b}"); //convert it to 5-bit binary

                    //Convert the 0's and 1's to random integers and push it to result
                    for c in char_bin.chars() {
                        let out = match c {
                            '0' => {
                                let mut rng = rand::thread_rng();
                                rng.gen_range(0..7) as u8 //up to, not including 7
                            }, _ => { // else 1
                                let mut rng = rand::thread_rng();
                                rng.gen_range(7..10) as u8 //up to, not including 10
                            }
                        };
                        result.push_str(&out.to_string());
                    };
                },
                c if c.is_lowercase() => {
                    let _char_dec = c as u8 as i32 - LOWERCASE_ASCII_OFFSET;
                    let char_bin = format!("{_char_dec:05b}");

                    //Convert the 0's and 1's to random integers and push it to result
                    for c in char_bin.chars() {
                        let out = match c {
                            '0' => {
                                let mut rng = rand::thread_rng();
                                rng.gen_range(0..7).to_string() //up to, not including 7
                            }, _ => { // else 1
                                let mut rng = rand::thread_rng();
                                rng.gen_range(7..10).to_string() //up to, not including 10
                            }
                        };
                        result.push_str(&out);
                    };
                },
                _ => {
                    result.push_str(&c.to_string());
                },
            };
        }
        result
    } else {  //decryption
        let mut number_counter = 0;
        let mut binary_dec = 0;
        for c in message.chars() {
            match c {
            c if c.is_whitespace() => {
                result.push_str(&c.to_string());
            },
            c if c.is_numeric() => {   
                let num = c as i32 - '0' as i32; //converts num to i32 from ascii representation
                number_counter += 1;
                if num >= 7 { //if number is high enough, it's a binary 1; add the converted binary value
                    let power_exp = 5 - number_counter as i32; //power exponent is 5-the counter since it reads L to R, not R to L
                    binary_dec += 2i32.pow(power_exp as u32); //convert it to dec
                }
                if number_counter >= 5 { //if counter if >=5, add the binary dec value and reset (since each # is a 5-bit binary)
                    number_counter = 0;
                    result.push_str(&((binary_dec + LOWERCASE_ASCII_OFFSET ) as u8 as char).to_string()); //get char as lowercase and return it
                    binary_dec = 0;
                }
            },
            _ => {
                return String::from("Cipher is not decryptable using a baconian cipher. Baconian ciphered text must contain only numeric digits and whitespace."); 
            }
            };   
        }
        result
    }
}

///Transpositional cipher that shuffles each character as though it is placed in a zig-zag pattern along a rail.
pub fn railfence_cipher(message: &str, rails: i32, enc_type: &str) -> String {
    let mut rail_matrix:Vec<Vec<char>> = vec![];
    let mut result = String::new();
    let mut cursor:usize = 0;
    enum Direction {UP,DOWN}
    let mut current_direction = Direction::DOWN;
    for _i in 0..rails {
        rail_matrix.push(vec![]); //add a row for each rail
    }
    if enc_type.contains("enc") {
        for c in message.chars() {
            rail_matrix[cursor].push(c);
            if matches!(current_direction, Direction::DOWN) { //checks for equality
                cursor += 1;
            } else {
                cursor -= 1;
            }

            if (cursor as i32) == (rails - 1) || cursor == 0 { //if at start or end of rails after incrementing or decrementing rail, change directions for next time
                if matches!(current_direction, Direction::DOWN) {current_direction = Direction::UP} else {current_direction = Direction::DOWN} //swap directions
            }
        }
        
           //Now we have the railmatrix made up, we can go through it in the correct order (row by row instead of column by column) and output it

        for i in 0..rails {
        for j in 0..rail_matrix[i as usize].len() {
            result += &(rail_matrix[i as usize][j as usize]).to_string() //add to result
        }
        }
    } else { //decryption 
        let mut rail_matrix = vec![vec![' ';message.chars().count()]; rails as usize];
        let (mut row_cursor, mut column_cursor) = (0,0);

        //First we mark each row to be filled
        for _c in message.chars() {
            if row_cursor == 0 {
                current_direction = Direction::UP;
            } else if row_cursor == (rails - 1) {
                current_direction = Direction::DOWN;
            }

            rail_matrix[row_cursor as usize][column_cursor as usize] = '*';
            column_cursor += 1;
            if matches!(current_direction,Direction::UP) {row_cursor += 1;} else {row_cursor -= 1};
        }



        //Converts the messasge to ascii, then slices it into an array of ascii characters (so it can be indexed properly)
        let indexed_message = AsciiStr::from_ascii(message).unwrap(); 
        let message_ascii_arr = indexed_message.as_slice();

        //Now we will the rail matrix with the correct items
        let mut message_cursor = 0;
        for i in 0..rails {
            for j in 0..message.chars().count() {
                if rail_matrix[i as usize][j as usize] == '*' && cursor < message.chars().count() {
                    rail_matrix[i as usize][j as usize] = message_ascii_arr[message_cursor].as_char();
                    message_cursor += 1;
                }
            }
        }

        //finally we can run through the matrix in a zig-zag pattern to reconstruct the original message.
        let mut message_array: Vec<char> = vec![];
        let (mut row_cursor, mut column_cursor) = (0,0);
        for _i in 0..message.chars().count() {
            if row_cursor == 0 {
                current_direction = Direction::UP;
            } else if row_cursor == (rails - 1) {
                current_direction = Direction::DOWN;
            }

            //This fills in spaces to the result
            if rail_matrix[row_cursor as usize][column_cursor as usize] != '*' {
                message_array.push(rail_matrix[row_cursor as usize][column_cursor as usize] as char);
                column_cursor += 1
            }
            
            if matches!(current_direction,Direction::UP) {row_cursor += 1;} else {row_cursor -= 1};
            result = message_array.clone().into_iter().collect();
        }
    }
    result //return result

}  

//Scores likelihood that a string is plain english and thus decoded, based on relative frequencies of letters, bigrams, trigrams, first characters, and last characters in words.
pub fn score_string(message: &str) -> f64 {
    let mut result_counts: Vec<i32> = vec![0;26];
    let mut result_weights: Vec<f64> = vec![0.0;26];
    let message = &message.to_lowercase();
    let char_count_total = message.chars().count() as i32;
    let mut diff_score = 0.0;
    for c in message.chars() {
        if c.is_alphabetic() {
            let current_char_int = ((c as u8) as i32) - LOWERCASE_ASCII_OFFSET;
            let char_count = result_counts[current_char_int as usize] + 1; //increment the count
            result_counts[current_char_int as usize] = char_count; //set the result count of the given char's int value to the new count
        }
        else {
            diff_score += 0.01
        }
    }
    for i in 0..result_counts.len() { //for each alphabetical letter we now get the weight
        result_weights[i as usize] = result_counts[i as usize] as f64 / char_count_total as f64;
    }

    //Now we have array result_counts containing a vector of weights for each character, in alphabetic order
    //Next we find the difference between each
    for i in 0..result_weights.len() {
        if result_weights[i as usize] != 0.0 {
            diff_score += (LETTER_LIKELIHOOD_DICT[i as usize] - result_weights[i as usize]).abs();
        }
    }
    diff_score
}

pub fn bruteforce(message: &str, enc_type: &str) -> String {
    let mut results: Vec<(f64, String)> = vec![]; //unwrap option or set to unknown encryption type

    if enc_type.contains("unk") || enc_type.contains("cae") { //unknown or caesar. Quick to test as there are only 26 possible shifts.
        let mut current: String;
        for i in 0..=26 { //0 to 26 inclusive
            current = caesar_cipher(message, i, "dec");
            results.push((score_string(&current), current)); //push data as tuple
        }
    }
    results.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
    for (score, message) in results {
        println!("\n"); //higher numbers are more different from english and thus less likely to be the plaintext result.
        println!("({:.2}): {}", score, message);
    }
    String::new()

}