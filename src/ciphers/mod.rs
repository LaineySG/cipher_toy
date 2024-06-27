use ascii::AsciiStr;
use core::cmp::Ordering;
use modinverse::modinverse;
use rand::Rng;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::time::Instant;
use std::io::Write;



const LOWERCASE_ASCII_OFFSET: i32 = 97;
const UPPERCASE_ASCII_OFFSET: i32 = 65;
const INTEGER_ASCII_OFFSET: i32 = 48; //48 is 0, 57 is 9
const LETTER_LIKELIHOOD: [f64;26] = [
    0.08167, 0.01492, 0.02782, 0.04253, 0.12702, 0.02228, 0.02015, 0.06094, 
    0.06966, 0.00153, 0.00772, 0.04025, 0.02406, 0.06749, 0.07507, 0.01929, 
    0.00095, 0.05987, 0.06327, 0.09056, 0.02758, 0.00978, 0.02360, 0.00150, 
    0.01974, 0.00074];

const FIRST_LETTER_LIKELIHOOD: [(char, f64);10] = [('t',0.1594),('a',0.1550),('i',0.0823),('s',0.0775),('o',0.0712),('c',0.0597),('m', 0.0426),('f',0.0408),('p',0.0400),('w',0.0382)]; //unwrap option or set to unknown encryption type
const LAST_LETTER_LIKELIHOOD: [(char, f64);10] = [('e',0.1917),('s',0.1435),('d',0.0923),('t',0.0864),('n',0.0786),('y',0.0730),('r', 0.0693),('o',0.0467),('l',0.0456),('f',0.0408)]; //unwrap option or set to unknown encryption type

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
    let mut rail_matrix:Vec<Vec<char>> = vec![];
    let mut result = String::new();
    result.push('\'');
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
        result.push('\'');
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
        let mut message_array: Vec<char> = vec!['\''];
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
        result.push('\'');
    }
    result //return result

}  

//Scores likelihood that a string is plain english and thus decoded, based on relative frequencies of letters, bigrams, trigrams, first characters, and last characters in words.
pub fn score_string(message: &str, word_list: &Vec<String>) -> f64 {
    let mut result_counts: Vec<i32> = vec![0;26];
    let mut result_weights: Vec<f64> = vec![0.0;26];
    let message = &message.to_lowercase();
    let char_count_total = message.chars().count() as i32;
    let mut diff_score = 0.0;
    let mut new_word:bool = true; //assume first char is a new word
    let mut previous_char: char = 'x'; //previous word is tracked for end of word letter comparisons
    //println!("CURRENT DIR: {}", current_dir().unwrap().display());
    for i in word_list {
        if message.contains(i) {
            //println!("Message contains {} so we are modifying diff by: {}\n",i, (-0.30 / char_count_total as f64 * 100.0));
            diff_score -= 0.30 / (char_count_total as f64); //If there are common english words then it's far more likely that it's an english output
        }
    }

    for c in message.chars() {
        if c.is_alphabetic() {
            let current_char_int = ((c as u8) as i32) - LOWERCASE_ASCII_OFFSET;
            let char_count = result_counts[current_char_int as usize] + 1; //increment the count
            result_counts[current_char_int as usize] = char_count; //set the result count of the given char's int value to the new count
            //println!("Message contains alphabetic'{}' so we are modifying diff by: {}\n",c, (-0.04 / char_count_total as f64 * 100.0));
            diff_score -= 0.02 / char_count_total as f64;
        }
        else { //If it's not alphabetic then it's more likely to be part of ciphertext, so slightly increment the different-ness for each non-alpha char.
        //println!("Message contains nonalphabetic '{}' so we are modifying diff by: {}\n",c, (0.15 / char_count_total as f64 * 100.0));
            diff_score += 0.15 / char_count_total as f64;
        }
        if new_word {
            for &(letter,score) in FIRST_LETTER_LIKELIHOOD.iter() {
                if letter == c {
                    //println!("Message word starts with common letter '{}' so we are modifying diff by: {}\n",c, (-score / char_count_total as f64 * 100.0));
                    diff_score -= score / char_count_total as f64;
                }
            }
        }
        if c.is_whitespace() {
            new_word = true; //next word will be a new word
            for &(letter,score) in LAST_LETTER_LIKELIHOOD.iter() {
                if letter == previous_char {
                    //println!("Message word ends with common letter '{}' so we are modifying diff by: {}\n",c, (-score / char_count_total as f64 * 100.0));
                    diff_score -= score / char_count_total as f64;
                }
            }
        } else {
            new_word = false;
        }; //set for next char whether it should be examined as beginning of word or not.
        previous_char = c;
    }
    for i in 0..result_counts.len() { //for each alphabetical letter we now get the weight
        result_weights[i as usize] = result_counts[i as usize] as f64 / char_count_total as f64;
    }

    //Now we have array result_counts containing a vector of weights for each character, in alphabetic order
    //Next we find the difference between each
    for i in 0..result_weights.len() {
        if result_weights[i as usize] != 0.0 {
            diff_score += (LETTER_LIKELIHOOD[i as usize] - result_weights[i as usize]).abs() * 2.0 / char_count_total as f64;
            //println!("Character number '{}' has a different weight from the norm, so we are modifying diff by: {}\n",i, ((LETTER_LIKELIHOOD[i as usize] - result_weights[i as usize]).abs() * 2.0 * 100.0 / char_count_total as f64));
        }
    }
    diff_score * 100.0
}

pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>> where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn bruteforce(message: &str, enc_type: &str) -> String {
    let mut wordlist: Vec<String> = vec![];
    if let Ok(lines) = read_lines("src/data/1000_most_common.txt") {
        // Consumes the iterator, returns an (Optional) String
        for line in lines.flatten() {
            wordlist.push(line);
        }
    } else {println!("Directory not found!")}

    let mut results: Vec<(f64, String, String)> = vec![]; //unwrap option or set to unknown encryption type
    let now = Instant::now();
    if enc_type.contains("unk") || enc_type.contains("cae") || enc_type.contains("cea") {
        println!("Checking caesar ciphers...");
        let mut current: String;
        for i in 0..=80 { //0 to 80 inclusive
            current = caesar_cipher(message, i, "dec");
            results.push((score_string(&current,&wordlist), current, "Caesar".to_string())); //push data as tuple
        }
    }
    if enc_type.contains("unk") || enc_type.contains("atb") {
        println!("Checking atbash cipher...");
        let current: String;
        current = atbash_cipher(message);
        results.push((score_string(&current,&wordlist), current, "Atbash".to_string())); //push data as tuple
    }
    if enc_type.contains("unk") || enc_type.contains("rot") || enc_type.contains("13") {
        println!("Checking ROT13 cipher...");
        let current: String;
        current = rot13_cipher(message);
        results.push((score_string(&current,&wordlist), current, "ROT13".to_string())); //push data as tuple
    }
    
    if enc_type.contains("unk") || enc_type.contains("aff"){
        println!("Checking Affine cipher...");
        let mut current: String;
        for a in 0..200 {
            match a {
                50 =>{println!("25%...")},
                100 =>{println!("50%...")},
                150 =>{println!("75%...")},
                185 =>{println!("90%...")},
                _ =>{},
            }
            for b in 0..26 {
                current = affine_cipher(message,a,b,"dec");
                results.push((score_string(&current,&wordlist), current, "Affine".to_string())); //push data as tuple
            }
        }
    }
    if enc_type.contains("unk") || enc_type.contains("bac") {
        println!("Checking Baconian cipher...");
        let current: String;
        current = baconian_cipher(message, "dec");
        results.push((score_string(&current,&wordlist), current, "Bacon".to_string())); //push data as tuple
    }
    if enc_type.contains("unk") || enc_type.contains("rai"){ 
        //railfence has a weird bug where it doesn't work unless 'rai' is specified, probably some kind of memory bug- fix later
        println!("Checking Railfence cipher...");
        for rails in 2..2000 {
            match rails {
                500 =>{println!("15%...")},
                1000 =>{println!("30%...")},
                1500 =>{println!("50%...")},
                2500 =>{println!("85%...")},
                _ =>{},
            }

            let current = railfence_cipher(message,rails,"dec");
            let temp = format!("Railfence[{}]",rails);
            results.push((score_string(&current,&wordlist), current, temp.clone())); //push data as tuple
        }
    }

    results.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));

    results.dedup_by_key(|k| k.1.clone()); //remove duplicates

    println!("Most likely results: ");
    println!("\n"); //higher numbers are more different from english and thus less likely to be the plaintext result.
    let output_file = File::create("bruteForceResults.txt").unwrap();
    for (score, message, type_of_cipher) in results.iter().take(50) {
        println!("({:.2}): {} [{}]\n", score, message.trim(), type_of_cipher.trim());
    }
    for (score, message, type_of_cipher) in results.iter() { 
        let line_str = format!("({:.2}): {} [{}]\n", score, message.trim(), type_of_cipher.trim());
        write!(&output_file, "{}", line_str).unwrap();
    }
    
    println!("\nFinished! Total time elapsed: {} seconds", now.elapsed().as_secs());
    String::new()

}

pub fn bruteforce_vigenere (message: &str) -> String {//unknown or caesar. Quick to test as there are only 26 possible shifts.
        println!("Checking vigenere ciphers...");
        let mut current: String;
        // for i in 0..=26 { //0 to 26 inclusive
        //     current = vigenere_cipher();
        //     results.push((score_string(&current,&wordlist), current, "Vigenere".to_string())); //push data as tuple
        // }
        return String::new();

}