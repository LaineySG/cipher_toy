use ascii::AsciiStr;
const LOWERCASE_ASCII_OFFSET: i32 = 97;

/// Shifts character by converting from char to u8, u8 to i32, add the i32 shift argument, then convert back to u8 and finally return as a character
pub fn shift_char(c: char, shift: i32) -> char {
    ((c as u8) as i32 + shift) as u8 as char
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
            println!("shift enc: {}", shift);
        } else if enc_type.contains("dec") {
            shift = -((key_ascii_arr[key_cursor] as i32) - LOWERCASE_ASCII_OFFSET);
            println!("shift dec: {}", shift);
        }

        key_cursor += 1;

        result += &shift_char(current_char, shift).to_string(); //Finally, add the shifted char as a string, to result.
    }
    result
}
