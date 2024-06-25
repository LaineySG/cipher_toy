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