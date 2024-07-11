
use core::cmp::Ordering;
use std::fs::File;
use std::io::{self, stdout, BufRead};
use std::path::Path;
use std::time::Instant;
use std::io::Write;
use std::sync::{Arc,Mutex};
use tokio::*;
use std::thread;
use crate::ciphers;

const LOWERCASE_ASCII_OFFSET: i32 = 97;
const LETTER_LIKELIHOOD: [f64;26] = [
    0.08167, 0.01492, 0.02782, 0.04253, 0.12702, 0.02228, 0.02015, 0.06094, 
    0.06966, 0.00153, 0.00772, 0.04025, 0.02406, 0.06749, 0.07507, 0.01929, 
    0.00095, 0.05987, 0.06327, 0.09056, 0.02758, 0.00978, 0.02360, 0.00150, 
    0.01974, 0.00074];
const AVG_ENGL_WORD_LENGTH: f64 = 4.7;
const FIRST_LETTER_LIKELIHOOD: [(char, f64);10] = [('t',0.1594),('a',0.1550),('i',0.0823),('s',0.0775),('o',0.0712),('c',0.0597),
('m', 0.0426),('f',0.0408),('p',0.0400),('w',0.0382)]; //unwrap option or set to unknown encryption type
const LAST_LETTER_LIKELIHOOD: [(char, f64);10] = [('e',0.1917),('s',0.1435),('d',0.0923),('t',0.0864),('n',0.0786),('y',0.0730),
('r', 0.0693),('o',0.0467),('l',0.0456),('f',0.0408)]; //unwrap option or set to unknown encryption type


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
    if new_word_counter == 0 {
        new_word_counter = 1;
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
    likelihood_of_english_score += 0.15 * (first_last_probability_score / (new_word_counter as f64)); //divide by the perfect score to get a ratio.
    
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

///Updates the results with a new message
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
            current = ciphers::caesar_cipher(message, i, "dec");
            results.push((score_string(&current,&wordlist), current, "Caesar".to_string())); //push data as tuple
        }
        updatepercentcompletion(percent_increment,completion_percentage_arcmutex.clone(),"add".to_string()); //adds to completion tally
    }
    if enc_type.contains("atbash") {
        update_results("Checking atbash cipher...".to_string(), result_arcmutex.clone());
        let current: String;
        current = ciphers::atbash_cipher(message);
        results.push((score_string(&current,&wordlist), current, "Atbash".to_string())); //push data as tuple
        updatepercentcompletion(percent_increment,completion_percentage_arcmutex.clone(),"add".to_string()); //adds to completion tally
    }
    if enc_type.contains("rot13") {
        update_results("Checking ROT13 cipher...".to_string(), result_arcmutex.clone());
        let current: String;
        current = ciphers::rot13_cipher(message);
        results.push((score_string(&current,&wordlist), current, "ROT13".to_string())); //push data as tuple
        updatepercentcompletion(percent_increment,completion_percentage_arcmutex.clone(),"add".to_string()); //adds to completion tally
    }
    if enc_type.contains("polybius") {
        update_results("Checking polybius cipher...".to_string(), result_arcmutex.clone());
        let current: String;
        current = ciphers::polybius_cipher(message,"dec");
        results.push((score_string(&current,&wordlist), current, "Polybius".to_string())); //push data as tuple
        updatepercentcompletion(percent_increment,completion_percentage_arcmutex.clone(),"add".to_string()); //adds to completion tally
    }
    if enc_type.contains("affine"){
        update_results("Checking affine cipher...".to_string(), result_arcmutex.clone());
        let mut current: String;
        for a in 0..=26 {
            for b in 0..=26 {
                current = ciphers::affine_cipher(message,a,b,"dec");
                results.push((score_string(&current,&wordlist), current, "Affine".to_string())); //push data as tuple
            }
        }
        updatepercentcompletion(percent_increment,completion_percentage_arcmutex.clone(),"add".to_string());
    }
    if enc_type.contains("baconian") {
        update_results("Checking baconian cipher...".to_string(), result_arcmutex.clone());
        let current: String;
        current = ciphers::baconian_cipher(message, "dec");
        results.push((score_string(&current,&wordlist), current, "Bacon".to_string())); //push data as tuple
        updatepercentcompletion(percent_increment,completion_percentage_arcmutex.clone(),"add".to_string()); //adds to completion tally
    }
    if enc_type.contains("railfence"){ 
        update_results("Checking railfence cipher...".to_string(), result_arcmutex.clone());
        for rails in 2..=message.len() {
            let current = ciphers::railfence_cipher(message,rails as i32,"dec");
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
            
            let keyed_cipher_arcmut = Arc::new(Mutex::new(keyed_cipher)); //cipher name stored for use during output.
            //arc allows it to be accessed for concurrent use.

            let keyedcipher_pct_arc = Arc::new(Mutex::new(keyedcipher_len as f32 * percent_increment));
            //Amount that the keyed cipher in question needs to fill

            let keycipher_result_arcmut: Arc<Mutex<Vec<(f64, String, String)>>> = Arc::new(Mutex::new(vec![]));
            //stores the results of the keyed cipher
        
            let progress_bar_lock = Arc::new(Mutex::new(false)); 
            //Used to lock the progress bar from updating if it has already been updated for a given percent (ie, 10.10 and 10.90 both floor() to 10 but we only want to update once at 10%.)

            let handles: Vec<_> = rock_you.chunks(1000).enumerate().map(|(_i,chunk)| {
                //Creates a list of thread handles, breaks the password list into pieces of length 1000 for easier concurrency.
                
                // Does some preliminary conversions. Clones the Arc data for safe access.
                let message = message.to_string();
                let chunk = chunk.to_vec(); //chunk of passwords to check

                //Arc::clones all the values from before so they can be safely used in the tasks
                let progress_bar_lock_arcmut_clone = Arc::clone(&progress_bar_lock);
                let result_arcmut_clone = Arc::clone(&keycipher_result_arcmut);
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

                    //pct tracks the percent based on the thread number. bruteforcelimit / 1000 is the chunk size. Multiplies by 100 to get as %.
                    let percent_threads = *thread_counter as f64 * 100.0 / (bruteforce_limit as f64 / 1000.0);
                    
                    let mutex_guard = keyedcipher_pct_arc_clone.lock().unwrap();  //This is the % that the keyed cipher in question has to fill
                    let mut progress_bar_lock_guard = progress_bar_lock_arcmut_clone.lock().unwrap();

                    if percent_threads.floor() % 4.0 <= 1.0 { //every 4%, update the progress bar with 1/25th of the total percentage allotted to the keyed cipher.
                        if !*progress_bar_lock_guard {
                            updatepercentcompletion(*mutex_guard / 25.0, completion_percentage_clone, "add".to_string());
                            *progress_bar_lock_guard = true;
                        }
                    } else {
                        *progress_bar_lock_guard = false; 
                    }
                    

                    print!("\r{}% done...", percent_threads.floor());
                    let _ = stdout().flush();
        
                    //Get the 1000 or 10000 most common words to give to score_string
                    if let Ok(lines) = read_lines(wordlist_path) {
                    for line in lines.flatten() {
                        wordlist.push(line);
                    }
                    } else {println!("Directory not found - common words!")}
        
                    //Results to be gathered later
                    let mut result_arcmut_clone_guard = result_arcmut_clone.lock().unwrap();


                    for j in 0..chunk.len() { //for each part of the chunk we attempt to cipher it then push the data to the results vector
                        let keyed_cipher_guard = keyed_cipher_arcmut_clone.lock().unwrap();
                        let current;
                        

                        let mut namecopy = keyed_cipher_guard.clone();
                        let upper_case_firstchar = namecopy.chars().next().expect("Error: Keyed cipher name not found!").to_uppercase();
                        let lower_case_name = namecopy.remove(0);
                        let keyed_cipher_name = format!("{}{} -",upper_case_firstchar,lower_case_name);

                        
                        if *keyed_cipher_guard == "autokey".to_string() { //run through the proper cipher
                            current = ciphers::autokey_cipher(&message,&chunk[j],"dec");

                        } else if *keyed_cipher_guard == "vigenere".to_string() {
                            current = ciphers::vigenere_cipher(&message,&chunk[j],"dec");

                        } else if *keyed_cipher_guard == "simplesub".to_string() {
                            current = ciphers::simplesub_cipher(&message,&chunk[j],"dec");

                        } else { //columnar
                            current = ciphers::col_trans_cipher(&message,&chunk[j],"dec");
                        }

                        result_arcmut_clone_guard.push((score_string(&current,&wordlist), current, (format!("{} {}",keyed_cipher_name,&chunk[j])))); 
                        //push data as tuple to results
                    }
        
                })
            }).collect(); //collect handles
        
            //joins the handles
            for handle in handles {
                handle.await.unwrap();
            }
            update_results("Collecting and joining results...".to_string(), result_arcmutex.clone());
            for (score, message, type_of_cipher) in keycipher_result_arcmut.lock().unwrap().iter() { 
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
