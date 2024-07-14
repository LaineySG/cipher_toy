use crate::utils;
use ascii::AsciiStr;
use modinverse::modinverse;
use rand::Rng;
use std::collections::HashMap;
use rand_seeder::Seeder;
use rand_pcg::Pcg64;
use rand::seq::SliceRandom;

const LOWERCASE_ASCII_OFFSET: i32 = 97;
const UPPERCASE_ASCII_OFFSET: i32 = 65;
const INTEGER_ASCII_OFFSET: i32 = 48; //48 is 0, 57 is 9
const POLYBIUS_SQUARE: [[char;5];5] = [['a','b','c','d','e'],['f','g','h','i','j'],['k','l','m','n','o'], ['p','q','r','s','t'],['u','v','w','x','y']];
const B64ARRAY: [char; 64] = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '+', '/'];


/// Caesar cipher shifts the values of each character in the message by a set amount, the shift key. To decrypt, it simply reverses this (shifting backwards).
pub fn caesar_cipher(message: &str, shift: i32, enc_type: &str) -> String {
    let mut result = String::new();
    let shift = if enc_type.contains("dec") { -shift } else { shift }; //If we're decrypting, shift should be backwards.
    for c in message.chars() { //For each character in the message to decrypt, we shift that char and push it to result
        result.push(utils::shift_char(c, shift));
    }
    result //Then return result
}

/// Vigenere cipher shifts the values of each character in the message by the value of a character in a repeating key. 
/// To decrypt, it simply reverses this (shifting backwards).
pub fn vigenere_cipher(message: &str, key: &str, enc_type: &str) -> String {
    let mut result = String::new();
    let mut key_cursor:usize = 0;
    let default:&str = "key";


        //Converts the key to ascii, then slices it into an array of ascii characters (so it's indexed properly)
        let indexed_key = AsciiStr::from_ascii(key); 
        let idx_key = match indexed_key {
            Ok(val) => val,
            Err(_e) => {
                AsciiStr::from_ascii(default).unwrap()
            }
        };

        let key_ascii_arr = idx_key.as_slice();
        
        if key_ascii_arr.len() == 0 {
            return String::new();
        }
        for (_idx, current_char) in message.chars().enumerate() { //returns index and char for each char in message.
            if key_cursor >= key_ascii_arr.len() { //If cursor is out of bounds (at the end of the key), reset it
                key_cursor = 0;
            }

            let mut shift = 0;

            //Grab the value of the key as an integer, subtract the base ascii offset for lowercase characters and save it as the shift value. 
            //Decrypt is same but shift becomes negative
            if enc_type.contains("enc") {
                shift = (key_ascii_arr[key_cursor] as i32) - LOWERCASE_ASCII_OFFSET;
            } else if enc_type.contains("dec") {
                shift = -((key_ascii_arr[key_cursor] as i32) - LOWERCASE_ASCII_OFFSET);
            }

            key_cursor += 1;
            //println!("Shifted to char: {}, {}", &shift_char(current_char, shift).to_string(), shift_char(current_char, shift) as u8);
            result += &utils::shift_char(current_char, shift).to_string(); //Finally, add the shifted char as a string, to result.
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
                    (((((c as u8 as i32) - UPPERCASE_ASCII_OFFSET) * a + (b % 26)) % 26) + UPPERCASE_ASCII_OFFSET) as u8 as char
                } else {
                    match modinverse(a, 26) {
                        Some(inverse_modulo) => {
                            let char_dec = (c as u8 as i32) - UPPERCASE_ASCII_OFFSET; //character value from 0 to 26   
        
                            (((inverse_modulo * (char_dec - (b % 26) + 26)) % 26 ) + UPPERCASE_ASCII_OFFSET) as u8 as char 
                            //undoes adding b, add 26 to ensure it's positive, multiply by inverse modulo (undoes modulo), takes mod 26 so the result will be between 0 and 25,
                            //and finally adds the offset and converts back to char
                        }
                        None => {
                            '/'
                        }
                    }
                }
            },
            c if c.is_lowercase() => {
                if enc_type.contains("enc") {
                    (((((c as u8 as i32) - LOWERCASE_ASCII_OFFSET) * a + (b % 26)) % 26) + LOWERCASE_ASCII_OFFSET) as u8 as char
                } else {
                    match modinverse(a, 26) {
                        Some(inverse_modulo) => {
                            let char_dec = (c as u8 as i32) - LOWERCASE_ASCII_OFFSET; //character value from 0 to 26                   
                            (((inverse_modulo * (char_dec - (b % 26) + 26)) % 26 ) + LOWERCASE_ASCII_OFFSET) as u8 as char
                        }
                        None => {
                            '/'
                        }
                    }
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
                result.push_str(&c.to_string());
            }
            };   
        }
        result
    }
}

///Transpositional cipher that shuffles each character as though it is placed in a zig-zag pattern along a rail.
pub fn railfence_cipher(message: &str, rails: i32, enc_type: &str) -> String {
    let message = &message.trim();
    let mut rail_matrix:Vec<Vec<char>> = vec![];
    let mut result = String::new();
    enum Direction {UP,DOWN}
    let mut current_direction = Direction::DOWN;

    for _i in 0..rails {
        rail_matrix.push(vec![]); //add a row for each rail
    }
    let mut cursor:usize = 0;
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
                if rail_matrix[i as usize][j as usize] == '*' && message_cursor < message.chars().count() {
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

///Substitutes characters based on a polybius 5x5 table one row down
pub fn polybius_cipher(message: &str, enc_type: &str) -> String {
    let message = &message.to_lowercase(); //turns message lowercase
    let mut result:String = String::new();
    if enc_type.contains("enc") {
        for c in message.chars() {
            let mut found:bool = false;
                for j in 0..5 {
                    if POLYBIUS_SQUARE[j].contains(&c) { //if that row of the square contains the character
                        found = true;

                        let index = POLYBIUS_SQUARE[j].iter().position(|&x| x == c).unwrap(); //checks for equality to find the index
                        if j >= 4 { //we don't want an out-of-bounds j for the index so we wrap the value
                            result += &POLYBIUS_SQUARE[0][index].to_string();
                        } else {
                            result += &POLYBIUS_SQUARE[j+1][index].to_string();
                        }
                }
            }
            if !found { //if non alphabetical char
                result += &c.to_string();
            }
        }
    } else { //decryption
        for c in message.chars() {
            let mut found = false;
            for j in 0..5 {
                if POLYBIUS_SQUARE[j].contains(&c) { //if that row of the square contains the character
                    found = true;

                    let index = POLYBIUS_SQUARE[j].iter().position(|&x| x == c).unwrap(); //checks for equality to find the index
                    if j == 0 { //we can't get a negative j for the index so we wrap the value
                        result += &POLYBIUS_SQUARE[4][index].to_string();
                    } else {
                        result += &POLYBIUS_SQUARE[j-1][index].to_string();
                    }
            }
            }
            if !found { //if non alphabetical char
                result += &c.to_string();
            }
        }
    }
    result
}

///Substitutes based on a random shuffle amount (a seed for randomization)
pub fn simplesub_cipher(message: &str,seed: &str,enc_type: &str) -> String {
    let mut alphabet: [char;26] = ['a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z'];
    let mut rng: Pcg64 = Seeder::from(seed).make_rng();
    alphabet.shuffle(&mut rng);
    let mut result = String::new();
    if enc_type.contains("enc") {
        for c in message.to_lowercase().chars() {
            if c.is_lowercase() {
                let ascii_decimal = (c as u8 as i32) - 97;
                if ascii_decimal < 26 {
                    result += &(alphabet[ascii_decimal as usize]).to_string();
                }
            } else {
                result += &c.to_string();
            }
        }
    } else {
        for c in message.to_lowercase().chars() {
            if c.is_lowercase() {                  
                let index = alphabet.iter().position(|&x| x == c).unwrap(); //get the index for character 'c'
                result += &((index as i32 + 97) as u8 as char).to_string();
            } else {
                result += &c.to_string();
            }
        }
    }
    result
}

///Columnal transpositional cipher, shifts columns in a table based on the keys to alphabetical order then reads vertically
pub fn col_trans_cipher(message: &str,mut key: &str,enc_type: &str) -> String {
    let mut result = String::new();
    let mut dict: HashMap<char, Vec<char>> = HashMap::new();
    if enc_type.contains("enc") {
        for c in key.chars() {
            // Create one column per character
            dict.insert(c, Vec::new()); // Set dictionary key with an empty array for now
        }
        let mut message:String = message.to_string();
        // Now table is made up with the correct columns equal to that of the key chars. Now we fill w/ values.
        let mut cursor = 0;
        while message.chars().count() % key.chars().count() != 0 {
            message += "*"; //add asterisk to fill in the extra spaces
        }
        for ch in message.chars() {
            // For each char in the message:
            dict.get_mut(&key.chars().nth(cursor).unwrap()).unwrap().push(ch); // Update the message for the key by adding a new vec entry
            cursor += 1;
            if cursor >= key.len() {
                cursor = 0;
            }
        }
        
        // Here is where we sort the dict keys alphabetically then push that to the result string
        let mut keylist: Vec<char> = dict.keys().cloned().collect();
        keylist.sort();

        for k in keylist {
            if let Some(v) = dict.get(&k) {
                result.push_str(&v.iter().collect::<String>());
            }
        }
    } else { // Decrypt message
        if key.chars().count() == 0 {
            eprintln!("ERROR: Key len of 0, using default key: 'key' instead.");
            key = "key";
        }
        let rows = (message.chars().count() + key.chars().count() - 1) / key.chars().count();
        for c in key.chars() {
            // Create one column per character
            dict.insert(c, Vec::new()); // Set dictionary key with an empty array for now
        }

        let keylist: Vec<char> = key.chars().collect();
        let mut kl_sorted = keylist.clone();
        kl_sorted.sort();

        // Fill the dictionary with the message characters
        let mut cursor = 0;
        let mut char_cursor = 0;
        for ch in message.chars() {
                dict.get_mut(&kl_sorted[char_cursor]).unwrap().push(ch);
                if cursor < (rows - 1) {cursor+=1;} else {cursor = 0; char_cursor += 1;}
        }
        //Now we must sort in the correct (original key's) order again to recover message 
        let mut returnedlist: HashMap<char, Vec<char>> = HashMap::new();
        for k in key.chars() { //for k in the OG key
            for (sorted_k, v) in &dict { //And for sorted k in the alphabetized key
                if &k == sorted_k {
                    returnedlist.insert(k, v.clone());
                    break;
                }
            }
        }
        //And push it to the results string
        for i in 0..rows {
            for k in key.chars() {
                if let Some(v) = returnedlist.get(&k) {
                    if i < v.len() {
                        result.push_str(&v[i].to_string());
                    }
                }
            }
        }
    }
    let output = format!("{}",result.clone());
    output
}

///Similar to vigenere but uses the messages itself to cipher following the key.
pub fn autokey_cipher(message: &str, key: &str, enc_type: &str) -> String {
    //encrypt: take key, add ciphertext, that's new key
    //decrypt: take key, decrypt first x letters, then add them to end, repeat until finished.
    let mut key_string; //This will store our computed key
    let mut result:String = String::new();
    let message = message.trim();
    let default:&str = "key";

    if enc_type.contains("enc") {
        key_string = format!("{}{}",key,message).to_lowercase();  //add key and message together

        //Converts the key to ascii, then slices it into an array of ascii characters (so it's indexed properly)
        let indexed_key = AsciiStr::from_ascii(&key_string); 
        let idx_key = match indexed_key {
            Ok(val) => val,
            Err(_e) => {
                eprintln!("ERROR: Key len of 0, using default key: 'key' instead.");
                AsciiStr::from_ascii(default).unwrap()
            }
        };

        let key_ascii_arr = idx_key.as_slice();
        
        if key_ascii_arr.len() == 0 {
            return String::new();
        }

        for (idx, current_char) in message.chars().enumerate() { //returns index and char for each char in message.    
            if key_ascii_arr[idx].is_alphabetic() {
                let shift = (key_ascii_arr[idx] as i32) - LOWERCASE_ASCII_OFFSET;
                result.push(utils::shift_char(current_char, shift)); //push the shifted char based on the shift from the key.
            } else {
                result.push(current_char);
            }
        }
        result
    } else {        

        let keylength = key.len();
        let msglength = message.chars().count();
        let num_of_chunks;
        if keylength == 0 {
            return String::new();
        } else {
            num_of_chunks = msglength / keylength;
        }
        

        for i in 0..=num_of_chunks { //for each chunk
            
            key_string = format!("{}{}",key,result).to_lowercase();  
            //Converts the key to ascii, then slices it into an array of ascii characters (so it's indexed properly)
            let indexed_key = AsciiStr::from_ascii(&key_string); 
            let idx_key = match indexed_key {
                Ok(val) => val,
                Err(_e) => {
                    AsciiStr::from_ascii(default).unwrap()
                }
            };
            let key_ascii_arr = idx_key.as_slice();
            if key_ascii_arr.len() == 0 {
                return String::new();
            }
            for (idx, current_char) in message.chars().enumerate() { //returns index and char for each char in message.
                if idx < (keylength + (i * keylength)) && idx >= (i * keylength) { //if between the message cursor and length
                    if idx < key_ascii_arr.len() {
                        if key_ascii_arr[idx].is_alphabetic() {
                            let shift = (key_ascii_arr[idx] as i32) - LOWERCASE_ASCII_OFFSET;
                            result.push(utils::shift_char(current_char, -shift));
                        } else {
                            result.push(current_char);
                        }
                    }
                }         
            }
        }  
        result
    }
}

///Converts to base64 and back. Not much of a cipher, very insecure.
pub fn base64_cipher(message: &str, enc_type: &str) -> String { 
    let mut result = String::new();
    if enc_type.contains("enc") {
        for c in message.chars() {
                    let _char_dec = c as u8 as i32; //Get the decimal char value
                    let char_bin = format!("{_char_dec:08b}"); //convert it to 8-bit binary
                    result.push_str(&char_bin);
        }
        while result.len() % 6 != 0 { //if not divisible by 6, pad with zeros
            result.push('0');
        }
        while result.len() % 24 != 0 { //If it's not divisible by 24, pad with '2' to get = later
            result.push('2');
        }

        //if the char is one digit, there will be 8 bits but we need a multiple of 6 bits. To achieve this, we will pad the final x bits with 0's, then pad with 2's until
        //we have a multiple of 6 digits (ie, 24 bits). The 2's will become =, as is customary for base64.

        //now we have binary stream, we go through 6 bits at a time and convert it to the base 64 value
        let mut number_counter = 0;
        let mut binary_dec = 0;
        let mut output = String::new();
        let mut padding_counter = 0;
        for c in result.chars() {
                let num = c as i32 - '0' as i32; //converts num to i32 from ascii representation
                number_counter += 1;
                if num == 1 {
                    let power_exp = 6 - number_counter as i32; //power exponent is 6-the counter since it reads L to R, not R to L
                    binary_dec += 2i32.pow(power_exp as u32); //convert it to dec

                } else if num == 2 { //2 is used to signify padding
                    padding_counter += 1;
                }
                if padding_counter >= 6 { //when padding item is found
                    output.push_str("=");
                    padding_counter = 0;
                    number_counter = 0;
                }
                if number_counter >= 6 { //if counter if >=6, add the binary dec value and reset (since each # is a 6-bit binary)
                    number_counter = 0;
                    output.push(B64ARRAY[binary_dec as usize]);
                    binary_dec = 0;
                }
        }
        output
    } else { //dec
        //First, we have the b64 string. We go through 1 char at a time and convert it to its 6 bit representation. We ignore the = padding.
        let mut binary_dec = 0;
        let mut output = String::new();
        for c in message.chars() {
            if c != '=' {
                for (idx, val) in B64ARRAY.iter().enumerate() {
                    if &c == val {
                        binary_dec = idx;
                    }
                }
                let _char_dec = binary_dec as u8 as i32; //Get the decimal char value as i32
                let char_bin = format!("{_char_dec:06b}"); //convert it to 6-bit binary
                output.push_str(&char_bin);
            }
        }
        while output.len() % 8 != 0 { //remove the 0's that were added to pad it to 6 bits. Then we will have the 8 bit representations
            output.pop();
        }
        //finally, convert the 8 bit representations back to decimal then back to ascii
        let mut number_counter = 0;
        let mut binary_dec = 0;
        for c in output.chars() {
            number_counter += 1;
            if c == '1' {
                let power_exp = 8 - number_counter as i32; //power exponent is 6-the counter since it reads L to R, not R to L
                binary_dec += 2i32.pow(power_exp as u32); //convert it to dec
            }
            if number_counter >= 8 {
                number_counter = 0;
                result.push(binary_dec as u8 as char);
                binary_dec = 0;
            }
        }
        result
    }
}

///Similar to vigenere cipher, but instead of plaintext + key % 26, it's key - plaintext % 26. 
/// Because of this, we can simply atbash to reverse the key then use the vigenere cipher.
pub fn beaufort_cipher(message: &str, key: &str, enc_type: &str) -> String {
    let reversed_key = atbash_cipher(key);
    let result = vigenere_cipher(message, &reversed_key, enc_type);
    result
}

//Ciphers to add:

//porta cipher
//playfair cipher
//four square cipher
//running key cipher
//ADFGX cipher
//bifid cipher
//fractionated morse code cipher
//hill cipher
//trifid cipher
//straddle checkerboard cipher
//homophonic substitution cipher
