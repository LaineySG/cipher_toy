use ascii::AsciiStr;
use core::cmp::Ordering;
use modinverse::modinverse;
use rand::Rng;
use std::fs::File;
use std::io::{self, stdout, BufRead};
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;
use std::io::Write;
use std::sync::{Arc,Mutex};
use tokio::*;
use rand_seeder::Seeder;
use rand_pcg::Pcg64;
use rand::seq::SliceRandom;
use std::thread;

const LOWERCASE_ASCII_OFFSET: i32 = 97;
const UPPERCASE_ASCII_OFFSET: i32 = 65;
const INTEGER_ASCII_OFFSET: i32 = 48; //48 is 0, 57 is 9
const LETTER_LIKELIHOOD: [f64;26] = [
    0.08167, 0.01492, 0.02782, 0.04253, 0.12702, 0.02228, 0.02015, 0.06094, 
    0.06966, 0.00153, 0.00772, 0.04025, 0.02406, 0.06749, 0.07507, 0.01929, 
    0.00095, 0.05987, 0.06327, 0.09056, 0.02758, 0.00978, 0.02360, 0.00150, 
    0.01974, 0.00074];
const AVG_ENGL_WORD_LENGTH: f64 = 4.7;

const FIRST_LETTER_LIKELIHOOD: [(char, f64);10] = [('t',0.1594),('a',0.1550),('i',0.0823),('s',0.0775),('o',0.0712),('c',0.0597),('m', 0.0426),('f',0.0408),('p',0.0400),('w',0.0382)]; //unwrap option or set to unknown encryption type
const LAST_LETTER_LIKELIHOOD: [(char, f64);10] = [('e',0.1917),('s',0.1435),('d',0.0923),('t',0.0864),('n',0.0786),('y',0.0730),('r', 0.0693),('o',0.0467),('l',0.0456),('f',0.0408)]; //unwrap option or set to unknown encryption type

const POLYBIUS_SQUARE: [[char;5];5] = [ //5 by 5 square for the polybius cipher
    ['a','b','c','d','e'],
    ['f','g','h','i','j'],
    ['k','l','m','n','o'], 
    ['p','q','r','s','t'],
    ['u','v','w','x','y']];


/// Shifts character while keeping it in a safe range of characters (stopping newline and other weird ascii chars as well as potential overflow)
pub fn shift_char(c: char, shift: i32) -> char {
    if (c as u8) < 48 || (c as u8) > 126 { //if it's a weird character don't shift it
        return c
    }
    let shifted_value = c as u8 as i32 + shift; //Shift the value. rem_euclid takes modulus basically, but for signed numbers. This keeps it in range of 48 to 126
    let wrapped_value = (shifted_value - 48).rem_euclid(79) + 48;
    wrapped_value as u8 as char
}

///Scores likelihood that a string is plain english and thus decoded, based on relative frequencies of letters, common english words, bigrams (not yet), trigrams (not yet), first characters, and last characters in words.
pub fn score_string(message: &str, word_list: &Vec<String>) -> f64 {
    
    let mut result_counts: Vec<i32> = vec![0;26]; //tracks count for each alphabetic character 
    let mut result_weights: Vec<f64> = vec![0.0;26]; //tracks weight relative to total char count, for each alphabetic character
    let mut first_char = true;
    let message = &message.trim().to_lowercase(); //turns message lowercase and trims whitespace

    let char_count_total = message.chars().count() as i32;

    let mut likelihood_of_english_score = 0.0; //likelihood of a message being english (as a percentage)

    let mut new_word:bool = true; //Tracks if current word is a new word, for first letter weight comparisons
    let mut previous_char: char = 'x'; //previous word is tracked for end of word letter comparisons

    //counts the # of found common words
    let mut counter = 0;
    for i in word_list {
        if message.contains(i) {
            counter += 1; 
        }
    }
    //Calculates the hit rate of common words per size of string
    let hit_rate = (counter as f64 * 3.0) / (char_count_total as f64); 
    likelihood_of_english_score += 0.20 * hit_rate;

    //alpha counter counts alphabetic chars, new_word_counter counts the # of total words. First and last_letter_likelihood track sum weights
    let mut alphacounter = 0;
    let mut new_word_counter = 0;
    let mut first_letter_likelihood = 0.0;
    let mut last_letter_likelihood = 0.0;
    let mut wordlength: i32 = 0;
    let mut wordlengths: Vec<i32> = vec![];
    let mut average_wordsize: f64 = 0.0;
    //for each alpha char, get the char value as an int (0 to 25), increment char count, and increment alpha counter
    for c in message.chars() {
        if c.is_alphabetic() {
            let current_char_int = ((c as u8) as i32) - LOWERCASE_ASCII_OFFSET;
            let char_count = result_counts[current_char_int as usize] + 1; //increment the count
            result_counts[current_char_int as usize] = char_count; //Set to incremented value
            alphacounter += 1; 
        } else if  first_char { //if first word is non-alphabetic, increment new word counter
            new_word_counter+=1;
            first_char = false;
        }

        //If new word is set then the previous char was whitespace or it's the start of the message. Get likelihood and sum the likelihood of each first letter.
        if new_word {
            for &(letter,score) in FIRST_LETTER_LIKELIHOOD.iter() {
                if letter == c {
                    first_letter_likelihood += score;
                }
            }
        }

        //If current char is whitespace, increment new_word counter, set new word to true for the next char so first letter can be examined.
        if c.is_whitespace() {
            new_word = true; //next word will be a new word
            new_word_counter+=1;
            for &(letter,score) in LAST_LETTER_LIKELIHOOD.iter() {
                //Since this is the start of a new word, get the previous char and compare it, and sum the last char likelihood.
                if letter == previous_char {
                    last_letter_likelihood += score;
                }
            }
            wordlengths.push(wordlength); //add word's length to counter
            wordlength = 0; //reset wordlength
        } else {
            new_word = false;
            wordlength += 1; //add 1 to wordlength counter
        };
        previous_char = c; //track previous char
    }

    wordlengths.push(wordlength); //add last word's length to counter after finishing the string

    for i in 0..wordlengths.len() {
        let mut length_diff = (wordlengths[i] as f64 - AVG_ENGL_WORD_LENGTH).abs(); //get the length diff
        length_diff = length_diff.powf(1.3);
        average_wordsize += length_diff
    }
    average_wordsize = average_wordsize / new_word_counter as f64;
    //smaller average wordsize diff is better (should be as close to 0 as possible)
    likelihood_of_english_score += 0.10 * (1.0 - (average_wordsize / 8.0)); //if average is over 8 diff it's unlikely to be english

    //More alphabetic chars = more likely to be english
    let alphabetic_rate = (alphacounter as f64) / (char_count_total as f64); 
    likelihood_of_english_score += 0.25 * alphabetic_rate;

    //Calculates first and last char based on the 'perfect score' for a word starting with and ending with the most common chars (t and e)
    let first_last_probability_score = (last_letter_likelihood + first_letter_likelihood) / 0.3511;
    likelihood_of_english_score += 0.15 * (first_last_probability_score / (new_word_counter as f64 - 1.0)); //divide by the perfect score to get a ratio.
    
    //for each alphabetical letter we now get the individual character counts adjusted for the total char count (this is the frequency)
    for i in 0..result_counts.len() {
        result_weights[i as usize] = result_counts[i as usize] as f64 / char_count_total as f64;
    }

    //Now we have array result_counts containing a vector of frequency weights for each character, in alphabetic order
    //Next we find the difference between each
    let mut letter_weight_diffs = 0.0;
    for i in 0..result_weights.len() {
            letter_weight_diffs += (LETTER_LIKELIHOOD[i as usize] - result_weights[i as usize]).abs(); 
            //a score of 1 means entirely different for all characters, 0 = perfect theoretical english
        
    }
    //Since 1 is entirely different, we take the complement because lower scores are better.
    likelihood_of_english_score += 0.30 * (1.0 - letter_weight_diffs);

    //Finally we convert to a % and return
    likelihood_of_english_score * 100.0
}

///Reads each line from a file
pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>> where P: AsRef<Path>, {
    let file = File::open(filename)?; //open the file and read the lines
    Ok(io::BufReader::new(file).lines())
}

///Updates the percent completion mutex; used during brute forces to update the completion percent for the GUI.
pub fn updatepercentcompletion(percent:f32,completion_percentage_arcmutex:Arc<Mutex<f32>>,update_type:String) {
    if update_type.contains("add") {
            let handle = thread::spawn(move || {
                let mut num = completion_percentage_arcmutex.lock().unwrap();
                if (*num + (percent as f32) / 100.0 * 360.0) <= 360.0 {
                    *num += (percent as f32) / 100.0 * 360.0;
                } else {
                    *num = 359.0;
                }
                if *num > 359.0 {
                    *num = 359.0;
                }
            });
            handle.join().unwrap();
    } else { //set
        let handle = thread::spawn(move || {
            let mut num = completion_percentage_arcmutex.lock().unwrap();
            *num = (percent as f32) / 100.0 * 360.0;
        });
        handle.join().unwrap();
    }
}

//Updates the results with a new message
pub fn update_results(results:String, result_arcmutex:Arc<Mutex<String>>) {
    let handle = thread::spawn(move || {
        let mut res = result_arcmutex.lock().unwrap();
        *res = results;
    });
    handle.join().unwrap();
}
 
///Attempts to brute force any cipher type except vigenere
pub async fn bruteforce(message: &str, enc_type: &str,completion_percentage_arcmutex:Arc<Mutex<f32>>,bruteforce_limit:i32,result_arcmutex:Arc<Mutex<String>>,wordlistbool: bool) -> io::Result<String> {
    let mut wordlist: Vec<String> = vec![];
    let mut output:String = String::new();    
    let mut keyedcipher_len: i32 = 0;
    let mut lengthcounter = enc_type.chars().filter(|c| *c == ',').count() as i32; //counter the number of things to check (used for the completion %)     
    if enc_type.contains("unknown") {lengthcounter -= 1;}


    let mut keyedciphers: Vec<String> = Vec::new();
    if enc_type.contains("autokey") {keyedciphers.push("autokey".to_string())};
    if enc_type.contains("vigenere") {keyedciphers.push("vigenere".to_string())};
    if enc_type.contains("columnar") {keyedciphers.push("columnar".to_string())};
    if enc_type.contains("simplesub") {keyedciphers.push("simplesub".to_string())};
    for _i in 0..keyedciphers.len() {
        lengthcounter -= 1;
        keyedcipher_len = (bruteforce_limit as f32 / 14344392.0 * 100.0 * 5.0) as i32;
        lengthcounter += keyedcipher_len;
    }

    let percent_increment = 1.0 / lengthcounter as f32 * 100.0; // 1 / the # of things to check.

    let wordlist_path: &str;
    if wordlistbool {
        wordlist_path = "src/data/10000_most_common.txt";
    } else {
        wordlist_path = "src/data/1000_most_common.txt";
    }
    if let Ok(lines) = read_lines(wordlist_path) {
        // Consumes the iterator, returns an (Optional) String
        for line in lines.flatten() {
            wordlist.push(line);
        }
    } else {println!("Directory not found!")}

    let mut results: Vec<(f64, String, String)> = vec![]; //unwrap option or set to unknown encryption type
    let now = Instant::now();

    if enc_type.contains("caesar") {
        update_results("Checking caesar ciphers...".to_string(), result_arcmutex.clone());
        let mut current: String;
        for i in 1..=80 { //0 to 80 inclusive
            current = caesar_cipher(message, i, "dec");
            results.push((score_string(&current,&wordlist), current, "Caesar".to_string())); //push data as tuple
        }
        updatepercentcompletion(percent_increment,completion_percentage_arcmutex.clone(),"add".to_string()); //adds to completion tally
    }
    if enc_type.contains("atbash") {
        update_results("Checking atbash cipher...".to_string(), result_arcmutex.clone());
        let current: String;
        current = atbash_cipher(message);
        results.push((score_string(&current,&wordlist), current, "Atbash".to_string())); //push data as tuple
        updatepercentcompletion(percent_increment,completion_percentage_arcmutex.clone(),"add".to_string()); //adds to completion tally
    }
    if enc_type.contains("rot13") {
        update_results("Checking ROT13 cipher...".to_string(), result_arcmutex.clone());
        let current: String;
        current = rot13_cipher(message);
        results.push((score_string(&current,&wordlist), current, "ROT13".to_string())); //push data as tuple
        updatepercentcompletion(percent_increment,completion_percentage_arcmutex.clone(),"add".to_string()); //adds to completion tally
    }
    if enc_type.contains("polybius") {
        update_results("Checking polybius cipher...".to_string(), result_arcmutex.clone());
        let current: String;
        current = polybius_cipher(message,"dec");
        results.push((score_string(&current,&wordlist), current, "Polybius".to_string())); //push data as tuple
        updatepercentcompletion(percent_increment,completion_percentage_arcmutex.clone(),"add".to_string()); //adds to completion tally
    }
    if enc_type.contains("affine"){
        update_results("Checking affine cipher...".to_string(), result_arcmutex.clone());
        let mut current: String;
        for a in 0..=26 {
            for b in 0..=26 {
                current = affine_cipher(message,a,b,"dec");
                results.push((score_string(&current,&wordlist), current, "Affine".to_string())); //push data as tuple
            }
        }
        updatepercentcompletion(percent_increment,completion_percentage_arcmutex.clone(),"add".to_string());
    }
    if enc_type.contains("baconian") {
        update_results("Checking baconian cipher...".to_string(), result_arcmutex.clone());
        let current: String;
        current = baconian_cipher(message, "dec");
        results.push((score_string(&current,&wordlist), current, "Bacon".to_string())); //push data as tuple
        updatepercentcompletion(percent_increment,completion_percentage_arcmutex.clone(),"add".to_string()); //adds to completion tally
    }
    if enc_type.contains("railfence"){ 
        update_results("Checking railfence cipher...".to_string(), result_arcmutex.clone());
        for rails in 2..=message.len() {
            let current = railfence_cipher(message,rails as i32,"dec");
            let temp = format!("Railfence[{}]",rails);
            let rail_msg = &current[1..current.len()-1]; //removes apostrophes from rail message before scoring 
            results.push((score_string(&rail_msg,&wordlist), current, temp.clone())); //push data as tuple
        }
        updatepercentcompletion(percent_increment,completion_percentage_arcmutex.clone(),"add".to_string());
    }
    if enc_type.contains("autokey") || enc_type.contains("vigenere") || enc_type.contains("simplesub") || enc_type.contains("columnar") { //password-cracking brute forces
         //gets list of common passwords to attempt to brute force. Also allows for limiting by bruteforce limit since the file is huge. Converts it to a vector for easy access.
        
        update_results("Loading password bruteforce list...".to_string(), result_arcmutex.clone());
        async fn get_password_list (bruteforce_limit:i32) -> io::Result<Vec<String>> {
            let mut password_list: Vec<String> = vec![];
            if let Ok(lines) = read_lines("src/data/rockyou.txt") {
                // Consumes the iterator, returns an (Optional) String
                for line in lines.flatten().take(bruteforce_limit.clone() as usize){ //take only the specified line count
                    password_list.push(line);
                }

            } else {println!("Directory not found - passwords!")}
            Ok(password_list)
        }
        
        let rock_you = match get_password_list(bruteforce_limit).await {
            Ok(list) => list,
            Err(e) => {
                eprintln!("There was an error reading the password list: {:?}", e);
                return Err(e)
            }
        };

        for keyed_cipher in keyedciphers {
            if keyed_cipher.contains("autokey") {
                update_results("Checking autokey cipher...".to_string(), result_arcmutex.clone());
            } else if keyed_cipher.contains("vigenere") {
                update_results("Checking vigenere cipher...".to_string(), result_arcmutex.clone());
            } else if keyed_cipher.contains("simplesub") {
                update_results("Checking simple substitution cipher...".to_string(), result_arcmutex.clone());
            } else if keyed_cipher.contains("columnar") {
                update_results("Checking columnar transposition cipher...".to_string(), result_arcmutex.clone());
            }


            let thread_counter = Arc::new(Mutex::new(0)); //thread counter, mutex allows us to lock or unlock it for mutually exclusive access
            
            let keyed_cipher_arcmut = Arc::new(Mutex::new(keyed_cipher));
            //arc allows it to be accessed for concurrent use.
            let keyedcipher_pct_arc = Arc::new(Mutex::new(keyedcipher_len as f32 * percent_increment));
            let keycipher_result_arcmutex: Arc<Mutex<Vec<(f64, String, String)>>> = Arc::new(Mutex::new(vec![]));
            //same as above but vector
        
            let mut tracker = Arc::new(Mutex::new(false));
            //Creates a list of thread handles, breaks the password list into pieces of length 1000 for easier concurrency.
            let handles: Vec<_> = rock_you.chunks(1000).enumerate().map(|(_i,chunk)| {
                
                // Does some preliminary conversions. Clones the Arc data for safe access.
                let message = message.to_string();
                let chunk = chunk.to_vec();     
                let tracker_arcmutex_clone = Arc::clone(&tracker);
                let result_arcmutex_clone = Arc::clone(&keycipher_result_arcmutex);
                let keyedcipher_pct_arc_clone = Arc::clone(&keyedcipher_pct_arc);
                let thread_counter = Arc::clone(&thread_counter);
                let completion_percentage_clone = Arc::clone(&completion_percentage_arcmutex);
                let keyed_cipher_arcmut_clone = Arc::clone(&keyed_cipher_arcmut);
        
                //Spawns tokio tasks to carry out the actual brute forcing
                task::spawn(async move {
        
                    let mut wordlist: Vec<String> = vec![];
                    
                    //Outputs the % finished amount
                    let mut thread_counter = thread_counter.lock().unwrap();
                    *thread_counter += 1;
                    let pct = *thread_counter as f64 * 100.0 / (bruteforce_limit as f64 / 1000.0);
                    
                    let mutex_guard = keyedcipher_pct_arc_clone.lock().unwrap();
                    let currently_done_pct= 100.0 - *mutex_guard as  f32;
                    let fraction_percent = (pct as f32) * *mutex_guard / 100.0;
                    let new_total_pct = currently_done_pct + fraction_percent;
                    //updatepercentcompletion(new_total_pct as i32, completion_percentage_clone, "set".to_string());
                    let mut tracker = tracker_arcmutex_clone.lock().unwrap();
                    if pct.floor() % 4.0 <= 1.0 {
                        if !*tracker {
                            updatepercentcompletion(*mutex_guard / 25.0, completion_percentage_clone, "add".to_string());
                            *tracker = true;
                        }
                    } else {
                        *tracker = false;
                    }
                    

                    print!("\r{}% done...", pct.floor());
                    let _ = stdout().flush();
        
                    //Get the 1000 most common words to give to score_string
                    if let Ok(lines) = read_lines(wordlist_path) {
                    for line in lines.flatten() {
                        wordlist.push(line);
                    }
                } else {println!("Directory not found - common words!")}
        
                //Lock results so we can access it, similar to counter earlier
                let mut result_arcmutex_clone_guard = result_arcmutex_clone.lock().unwrap();
                for j in 0..chunk.len() { //for each part of the chunk we attempt to cipher it then push the data to the results vector
                    let keyed_cipher_guard = keyed_cipher_arcmut_clone.lock().unwrap();
                    let current;
                    if *keyed_cipher_guard == "autokey".to_string() {
                        current = autokey_cipher(&message,&chunk[j],"dec");
                        result_arcmutex_clone_guard.push((score_string(&current,&wordlist), current, (format!("Autokey - {}",&chunk[j])))); //push data as tuple

                    } else if *keyed_cipher_guard == "vigenere".to_string() {
                        current = vigenere_cipher(&message,&chunk[j],"dec");
                        result_arcmutex_clone_guard.push((score_string(&current,&wordlist), current, (format!("Vigenere - {}",&chunk[j])))); //push data as tuple

                    } else if *keyed_cipher_guard == "simplesub".to_string() {
                        current = simplesub_cipher(&message,&chunk[j],"dec");
                        result_arcmutex_clone_guard.push((score_string(&current,&wordlist), current, (format!("Simplesub - {}",&chunk[j])))); //push data as tuple

                    } else if *keyed_cipher_guard == "columnar".to_string() {
                        current = col_trans_cipher(&message,&chunk[j],"dec");
                        result_arcmutex_clone_guard.push((score_string(&current,&wordlist), current, (format!("Columnar - {}",&chunk[j])))); //push data as tuple

                    }
        
                }
        
                })
            }).collect();
        
            //joins the handles
            for handle in handles {
                handle.await.unwrap();
            }
            update_results("Collecting and joining results...".to_string(), result_arcmutex.clone());
            for (score, message, type_of_cipher) in keycipher_result_arcmutex.lock().unwrap().iter() { 
                results.push((*score, message.to_string(), type_of_cipher.to_string())); //push data as tuple
            }
            
        }
    }



    update_results("Sorting results...".to_string(), result_arcmutex.clone());
    results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal)); //Do a comparison to sort and get the best results.

    results.dedup_by_key(|k| k.1.clone()); //remove duplicates

    println!("Most likely results: ");
    println!("\n"); //higher numbers are more different from english and thus less likely to be the plaintext result.

    let output_file = File::create("bruteForceResults.txt").unwrap();
    for (score, message, type_of_cipher) in results.iter().take(50) { //put the top 50 most likely results in the chat
        println!("({:.2}): {} [{}]\n", score, message.trim(), type_of_cipher.trim());
        output = format!("{}({:.2}): {} [{}]\n\n",output, score, message.trim(), type_of_cipher.trim());
    }
    for (score, message, type_of_cipher) in results.iter() {  //put the other attempts in the brute force results file
        let line_str = format!("({:.2}): {} [{}]\n", score, message.trim(), type_of_cipher.trim());
        write!(&output_file, "{}", line_str).unwrap();
    }
    updatepercentcompletion(100.0, completion_percentage_arcmutex.clone(), "set".to_string());

    output = format!("\nFinished! Total time elapsed: {} seconds\n\n", now.elapsed().as_secs()) + &output;
    Ok(output)

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
            eprintln!("ERROR: Key len of 0, key: {}",key);
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
                result.push(shift_char(current_char, shift)); //push the shifted char based on the shift from the key.
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
                            result.push(shift_char(current_char, -shift));
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

//Ciphers to add:
//beaufort cipher
//porta cipher
//running key cipher
//homophonic substitution cipher
//four square cipher
//hill cipher
//playfair cipher
//ADFGX cipher
//bifid cipher
//straddle checkerboard cipher
//trifid cipher
//base64 cipher
//fractionated morse code cipher
