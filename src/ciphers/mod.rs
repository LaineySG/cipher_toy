use ascii::AsciiStr;
const LOWERCASE_ASCII_OFFSET: i32 = 97;
pub fn shift_char(c: char, shift: i32) -> char { 
    //shifts char by converting the character to an 8bit int, then to a 32bit int, adds the shift integer value
    //converts the new value back to an 8 bit int, and finally to a char. Returns the new, shifted char.
    ((c as u8) as i32 + shift) as u8 as char
}

pub fn caesar_cipher(message: &str, shift: i32, enc_type: &str) -> String {
    //applies the caesar cipher based on the input message string, type of encryption, and shift integer value

    let mut result = String::new();
    let shift = if enc_type.contains("dec") { -shift } else { shift }; //If we're decrypting, shift should be backwards.
    for c in message.chars() { //For each character in the message to decrypt, we shift that char and push it to result
        result.push(shift_char(c, shift));
    }
    result //Then return result
}

pub fn vigenere_cipher(message: &str, key: &str, enc_type: &str) -> String {
    let mut result = String::new();
    let mut key_cursor:usize = 0;
    for (_idx, current_char) in message.chars().enumerate() { //returns index and char for each char in message.
        if key_cursor >= key.chars().count() { //If cursor is out of bounds (at the end of the key)
            key_cursor = 0;    
        }
        let indexed_key = AsciiStr::from_ascii(key).unwrap();
        let key_ascii_arr = indexed_key.as_slice();
        

        let mut shift = 0;
        if enc_type.contains("enc") {
            shift = (key_ascii_arr[key_cursor] as i32) - LOWERCASE_ASCII_OFFSET;
            println!("shift enc: {}", shift);
        } else if enc_type.contains("dec") {
            shift = -((key_ascii_arr[key_cursor] as i32) - LOWERCASE_ASCII_OFFSET);
            println!("shift dec: {}", shift);
        }
        key_cursor += 1;
        result += &shift_char(current_char, shift).to_string();
    }
    result
}
