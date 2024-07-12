//#![windows_subsystem = "windows"] //hides windows terminal by default since it's not necessary w/ the GUI.

mod ciphers;
mod utils;
use eframe::egui;
use egui::Color32;
use core::fmt;
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(),eframe::Error> {
    eframe::run_native("Cipher Toy", eframe::NativeOptions::default(), Box::new(|cc| Box::new(MainWindow::new(cc))))
}
struct MainWindow {
    message_input:String,
    int_a:i32,
    int_b:i32,
    float_percent:f64,
    key_input:String,    
    wordlist:bool,
    completion_percentage_arcmutex: Arc<Mutex<f32>>,
    completion_progress: f32,
    selected_action:SelectedActionEnum,
    encrypt_or_decrypt:EncOrDec, //True will be encrypt
    result:Arc<Mutex<String>>,
    bruteforce_selections:HashMap<String,bool>,
}

#[derive(Debug, PartialEq)]
enum SelectedActionEnum {
    Caesar,Vigenere,Atbash,Affine,Baconian,Polybius,SimpleSub,RailFence,Rot13,Bruteforce,Score,Autokey,Columnar,Base64
}
impl fmt::Display for SelectedActionEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
#[derive(Debug, PartialEq)]
enum EncOrDec {
    Encrypt, Decrypt, Other
}

impl fmt::Display for EncOrDec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl MainWindow {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            message_input: String::new(),
            int_a: 1,
            int_b: 1,
            float_percent: 5.0,
            completion_progress: 0.0,
            key_input:String::new(),    
            completion_percentage_arcmutex: Arc::new(Mutex::new(0.0)),
            selected_action: SelectedActionEnum::Caesar,
            encrypt_or_decrypt: EncOrDec::Encrypt,
            result: Arc::new(Mutex::new(String::new())),
            wordlist: false, //false = 1000
            bruteforce_selections: HashMap::from([
                ("unknown".to_string(), false),
                ("caesar".to_string(), false),("simplesub".to_string(), false),
                ("autokey".to_string(), false),("atbash".to_string(), false),
                ("affine".to_string(), false),("railfence".to_string(), false),
                ("baconian".to_string(), false),("polybius".to_string(), false),
                ("rot13".to_string(), false), ("vigenere".to_string(), false),
                ("columnar".to_string(), false),("base64".to_string(),false),
            ]),
        }
    }
    fn call_run_operations(result: Arc<Mutex<String>>,message_input: String, selected_action: String, key_input: String, encrypt_or_decrypt: String,completion_percentage_arcmutex:Arc<Mutex<f32>>,bruteforce_options:HashMap<String,bool>,wordlist:bool) {
        let _handle = tokio::spawn(async move {
            let _output = run_operations(message_input.to_string(), selected_action.to_string(),key_input.to_string(), encrypt_or_decrypt.to_string(),completion_percentage_arcmutex,result,bruteforce_options,wordlist).await;

        });
        
    }
}

impl eframe::App for MainWindow {
   fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let Self {message_input,selected_action,mut completion_progress,completion_percentage_arcmutex,
                encrypt_or_decrypt,result,wordlist, key_input,int_a,int_b,float_percent,bruteforce_selections} = self;
        
        

            egui::SidePanel::right("right_panel")
                .resizable(true)
                .default_width(400.0)
                .width_range(150.0..=400.0)
                .show_inside(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Info");
                    });
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let infoblock = get_info(selected_action.to_string());
                        ui.label(format!("{infoblock}"));
                    });
                });
            
            ui.heading("Cipher Toy");
            ui.separator();
            ui.label("Input text");
            ui.text_edit_multiline(message_input);
            ui.separator();
            egui::ComboBox::from_label("Cipher Type")
                .selected_text(format!("{:?}", selected_action))
                .show_ui(ui, |ui| {
                    ui.selectable_value(selected_action, SelectedActionEnum::Caesar, "Caesar Cipher");
                    ui.selectable_value(selected_action, SelectedActionEnum::Vigenere, "Vigenere Cipher");
                    ui.selectable_value(selected_action, SelectedActionEnum::Atbash, "Atbash Cipher");
                    ui.selectable_value(selected_action, SelectedActionEnum::Affine, "Affine Cipher");
                    ui.selectable_value(selected_action, SelectedActionEnum::Baconian, "Baconian Cipher");
                    ui.selectable_value(selected_action, SelectedActionEnum::Polybius, "Polybius Cipher");
                    ui.selectable_value(selected_action, SelectedActionEnum::SimpleSub, "Simple Substitution Cipher");
                    ui.selectable_value(selected_action, SelectedActionEnum::RailFence, "Railfence Cipher");
                    ui.selectable_value(selected_action, SelectedActionEnum::Rot13, "ROT13 Cipher");
                    ui.selectable_value(selected_action, SelectedActionEnum::Autokey, "Autokey Cipher");
                    ui.selectable_value(selected_action, SelectedActionEnum::Base64, "Base64 Cipher");
                    ui.selectable_value(selected_action, SelectedActionEnum::Columnar, "Columnar Transpositional Cipher");
                    ui.selectable_value(selected_action, SelectedActionEnum::Bruteforce, "Bruteforce");
                    ui.selectable_value(selected_action, SelectedActionEnum::Score, "Score String");
                });
            ui.separator();
            match selected_action.to_string().to_lowercase() {
                x if x.contains("simplesub") || x.contains("vigenere") || x.contains("autokey") || x.contains("column") => {
                    ui.label("Secret Key");
                    ui.text_edit_singleline(key_input);
                    ui.separator();
                }
                x if x.contains("affine") => {
                    ui.label("Secret Key a");
                    ui.add(
                        egui::Slider::new(int_a,1..=25).step_by(2.0)
                    );
                    if *int_a == 13 {
                        ui.colored_label(Color32::RED, "13 is not coprime to 26!");
                    }
                    
                    ui.label("Secret Key b");
                    ui.add(
                        egui::DragValue::new(int_b).clamp_range(1..=26)
                    );
                    *key_input = format!("{},{}",int_a.to_string(),int_b.to_string());
                    ui.separator();
                }
                x if x.contains("railfence") => {
                    ui.label("Secret Key");
                    ui.add(
                        egui::DragValue::new(int_a).clamp_range(2..=message_input.len())
                    );
                    *key_input = int_a.to_string();
                    ui.separator();
                }
                x if x.contains("caesar") => {
                    ui.label("Secret Key");
                    ui.add(
                        egui::DragValue::new(int_a).clamp_range(1..=80)
                    );
                    *key_input = int_a.to_string();
                    ui.separator();
                }
                x if x.contains("score") => {
                    {
                        ui.label("Wordlist for scoring (longer = better but slower)");
                        ui.horizontal(|ui| {
                            ui.checkbox(wordlist,"");
                        });
                        if *wordlist {
                            ui.label("10,000 words");
                        } else {
                            ui.label("1000 words");
                        }
                    }
                }
                x if x.contains("bruteforce") => {
                    ui.vertical_centered(|ui| {
                        ui.horizontal(|ui| {
                            if ui.checkbox(bruteforce_selections.get_mut("unknown").expect("Not found!"), "Check all").changed() {
                                if *bruteforce_selections.get_mut("unknown").expect("Checkbox not found!") == true {
                                    for (_k,v) in bruteforce_selections.into_iter() {
                                        *v = true;
                                    }
                                } else {
                                    for (_k,v) in bruteforce_selections.into_iter() {
                                        *v = false;
                                    }
                                }
                            };
                            ui.checkbox(bruteforce_selections.get_mut("caesar").expect("Not found!"), "Caesar");
                            ui.checkbox(bruteforce_selections.get_mut("simplesub").expect("Not found!"), "*SimpleSub");
                            ui.checkbox(bruteforce_selections.get_mut("autokey").expect("Not found!"), "*Autokey");
                            ui.checkbox(bruteforce_selections.get_mut("base64").expect("Not found!"), "Base64");
                        });
                        ui.horizontal(|ui| {
                            ui.checkbox(bruteforce_selections.get_mut("atbash").expect("Not found!"), "Atbash");
                            ui.checkbox(bruteforce_selections.get_mut("affine").expect("Not found!"), "Affine");
                            ui.checkbox(bruteforce_selections.get_mut("railfence").expect("Not found!"), "Railfence");
                            ui.checkbox(bruteforce_selections.get_mut("vigenere").expect("Not found!"), "*Vigenere");
                        });
                        ui.horizontal(|ui| {
                            ui.checkbox(bruteforce_selections.get_mut("baconian").expect("Not found!"), "Baconian");
                            ui.checkbox(bruteforce_selections.get_mut("polybius").expect("Not found!"), "Polybius");
                            ui.checkbox(bruteforce_selections.get_mut("rot13").expect("Not found!"), "ROT13");
                            ui.checkbox(bruteforce_selections.get_mut("columnar").expect("Not found!"), "*Columnar Transposition");
                        });
                    });
                    if *bruteforce_selections.get_mut("vigenere").expect("Not found!") == true ||
                    *bruteforce_selections.get_mut("autokey").expect("Not found!") == true ||
                    *bruteforce_selections.get_mut("columnar").expect("Not found!") == true ||
                    *bruteforce_selections.get_mut("simplesub").expect("Not found!") == true //these are the keyed-ciphers
                    {
                        ui.label("% of words to check");
                        ui.add(    
                            egui::DragValue::new(float_percent).clamp_range(1.0..=100.0)
                        );
                        *key_input = float_percent.to_string();

                    }
                    {
                        ui.label("Wordlist for scoring (longer = better but slower)");
                        ui.horizontal(|ui| {
                            ui.checkbox(wordlist,"");
                        });
                        if *wordlist {
                            ui.label("10,000 words");
                        } else {
                            ui.label("1000 words");
                        }
                    }
                    ui.separator();
                }
                _ => {}
            }
            if !selected_action.to_string().to_lowercase().contains("bruteforce") && !selected_action.to_string().to_lowercase().contains("score")
            && !selected_action.to_string().to_lowercase().contains("atbash") && !selected_action.to_string().to_lowercase().contains("rot13") {
                ui.horizontal(|ui| {
                    ui.radio_value(encrypt_or_decrypt, EncOrDec::Encrypt, "Encrypt");
                    ui.radio_value(encrypt_or_decrypt, EncOrDec::Decrypt, "Decrypt");
                });
                ui.separator();
            } else {
                *encrypt_or_decrypt = EncOrDec::Other;
            }
            ui.vertical_centered(|ui| {
                if ui.button("Start").clicked() {
                    *result.clone().lock().unwrap() = "Working...".to_string();

                    MainWindow::call_run_operations(result.clone(),message_input.to_string(), selected_action.to_string(),
                    key_input.to_string(), encrypt_or_decrypt.to_string(),
                    completion_percentage_arcmutex.clone(),bruteforce_selections.clone(),wordlist.clone());

                }
             });
            
            let result_description = match encrypt_or_decrypt.to_string().to_lowercase() {
                x if x.contains("enc") => "ciphertext",
                x if x.contains("dec") => "plaintext",
                _=> "output", //Other - for example atbash
            };

            egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(false)
            .min_height(400.0)
            .default_height(400.0)
            .show_inside(ui, |ui| { egui::ScrollArea::vertical().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    let res_string = result.lock().unwrap().clone();
                    ui.label(format!("Resulting {} is: \t",result_description));

                    ui.label(format!("{res_string}")).highlight();
                    completion_progress = completion_percentage_arcmutex.lock().unwrap().clone() as f32;

                    if completion_progress > 0.0 {
                        let progress = completion_progress / 360.0;
                        let progress_bar = egui::ProgressBar::new(progress).show_percentage();
                        ui.add(progress_bar);
                    }
                    if completion_progress >= 360.0 { //reset
                        *completion_percentage_arcmutex.lock().unwrap() = 0.0;
                    }
                });
            });
            });
        });
    }
}

///main operation running logic
async fn run_operations(message_input:String,selected_action:String,secret_key:String,mut encrypt_or_decrypt:String,completion_percentage_arcmutex:Arc<Mutex<f32>>,result:Arc<Mutex<String>>, bruteforce_options: HashMap<String,bool>,wordlist: bool) -> String {
    
    if message_input.len() < 1 {
        return "Error: Message not found! Ensure that message is not empty.".to_string();
    }
    
    encrypt_or_decrypt = encrypt_or_decrypt.to_lowercase();
    let x = match selected_action.to_lowercase() {
        opt if opt.contains("caesar") => {
            if secret_key.trim().to_lowercase().parse::<i32>().is_ok() { 
                let shift_key = secret_key.trim().to_lowercase().parse::<i32>().unwrap(); //Try to get shift key as integer
                let result = ciphers::caesar_cipher(&message_input,shift_key,&encrypt_or_decrypt);
                result
            } else {
                String::from("Error: Ensure that the shift key is a valid integer.")
            }
        },
        opt if opt.contains("vigenere") => {
            let result = ciphers::vigenere_cipher(&message_input, &secret_key, &encrypt_or_decrypt);
            result
        },
        opt if opt.contains("atbash") => {
            let result = ciphers::atbash_cipher(&message_input);
            result
        },
        opt if opt.contains("rot13") => {
            let result = ciphers::rot13_cipher(&message_input);
            result
        },
        opt if opt.contains("affine") => {
            let args: Vec<&str> = secret_key.split(',').collect();
            if let Some(_val) = args.get(1) {
                if args[0].parse::<i32>().is_ok() && args[1].parse::<i32>().is_ok() {
                    let a = args[0].trim().to_lowercase().parse::<i32>().unwrap(); 
                    let b = args[1].trim().to_lowercase().parse::<i32>().unwrap(); 
                    let result = ciphers::affine_cipher(&message_input,a,b,&encrypt_or_decrypt);
                    result
                } else {
                    let result = String::from("Error: Key for the affine cipher could not be parsed. Ensure that 'a' and 'b' are both integers.");
                    result
                }
            } else {String::from("Error: key for Affine cipher could not be parsed. Ensure that 'a' and 'b' are both selected, valid integers.")}
        },
        opt if opt.contains("bacon") => {
            let result = ciphers::baconian_cipher(&message_input, &encrypt_or_decrypt);
            result
        },
        opt if opt.contains("railfence") => {
            if secret_key.parse::<i32>().is_ok() {
                let key_int = secret_key.trim().to_lowercase().parse::<i32>().unwrap(); 
                let result = ciphers::railfence_cipher(&message_input, key_int, &encrypt_or_decrypt);
                result
            } else {
                String::from("Error: For the railfence cipher, the secret key must be an integer!")
            }
        },
        opt if opt.contains("base64") => {
            let result = ciphers::base64_cipher(&message_input, &encrypt_or_decrypt);
            result
        },
        opt if opt.contains("autokey") => {
            let result = ciphers::autokey_cipher(&message_input, &secret_key, &encrypt_or_decrypt);
            result
        },
        opt if opt.contains("polybius") => {
            let result = ciphers::polybius_cipher(&message_input, &encrypt_or_decrypt);
            result
        },
        opt if opt.contains("simplesub") => {
            let result = ciphers::simplesub_cipher(&message_input, &secret_key, &encrypt_or_decrypt);
            result
        },
        opt if opt.contains("columnar") => {
            let result = ciphers::col_trans_cipher(&message_input, &secret_key, &encrypt_or_decrypt);
            result
        },
        opt if opt.contains("score") => {
            if wordlist == true { //10000
                let mut word_list: Vec<String> = vec![];
                if let Ok(lines) = utils::read_lines("src/data/10000_most_common.txt") {
                    // Consumes the iterator, returns an (Optional) String
                    for line in lines.flatten() {
                        word_list.push(line);
                    }
                    let result: f64 = utils::score_string(&message_input, &word_list);
                    result.to_string()
                } else {String::from("Error: Word list directory not found! Ensure that 10000_most_common.txt can be found in the src/data folder.")}

            } else {
                let mut word_list: Vec<String> = vec![];
                if let Ok(lines) = utils::read_lines("src/data/1000_most_common.txt") {
                    // Consumes the iterator, returns an (Optional) String
                    for line in lines.flatten() {
                        word_list.push(line);
                    }
                    let result = utils::score_string(&message_input, &word_list);
                    result.to_string()
                } else {String::from("Error: Word list directory not found! Ensure that 1000_most_common.txt can be found in the src/data folder.")}

            }
        },
        opt if opt.contains("bruteforce") => {
            let mut bruteforce_options_string = String::new();
            for (k, v) in bruteforce_options.iter() {
                if *v == true {
                    bruteforce_options_string += k;
                    bruteforce_options_string += ",";
                }
            }
            let mut bfl = 0;
            if bruteforce_options_string.contains("vigenere") || bruteforce_options_string.contains("columnar") || 
            bruteforce_options_string.contains("autokey") || bruteforce_options_string.contains("simplesub") {
                if secret_key.parse::<f64>().is_ok() {
                    let keyasf64 = secret_key.trim().to_lowercase().parse::<f64>().unwrap();
                    bfl = (keyasf64 / 100.0 * 14344392.0).floor() as i32; //14344392 is the number of passwords in the bruteforce list
                }
            }
            
            let result = utils::bruteforce(&message_input, &bruteforce_options_string,completion_percentage_arcmutex,bfl,result.clone(),wordlist).await;
            if result.is_ok() {
                result.unwrap()
            } else {
                String::from("Error: bruteforce could not be completed.")
            }
        },
        _ => {
            String::from("Error: No action was selected!")
        }
    };
    let result_clone = Arc::clone(&result);
    let handle = thread::spawn(move || {
        let mut res_mod = result_clone.lock().unwrap();
        *res_mod = x;
    });
    handle.join().unwrap();
    String::new()
}

fn get_info(selected_action:String) -> String {
    match selected_action.to_lowercase() {
        opt if opt.contains("caesar") => {
            String::from("A caesar cipher is a common monoalphabetic substitution cipher that shifts letters by a key called the shift value.")
        },
        opt if opt.contains("vigenere") && !opt.contains("bruteforce") => {
            String::from("A vigenere cipher is a common polyalphabetic substitution cipher that shifts letters by the values of a repeating key.")
        },
        opt if opt.contains("atbash") => {
            String::from("An Atbash cipher is a common monoalphabetic substitution cipher that reverses the characters in a message.")
        },
        opt if opt.contains("rot13") => {
            String::from("An ROT13 cipher is a simple substitution cipher that rotates each character by 13 spaces, as if choosing the other side of an alphabet wheel.")
        },
        opt if opt.contains("affine") => {
            String::from("An affine cipher is a monoalphabetic substitution cipher that performs a mathematical operation, *a + b on a character, given the key [a,b]. a must be coprime to 26 (eg 3,5,7,9,11,15...).")
        },
        opt if opt.contains("bacon") => {
            String::from("A baconian cipher is a monoalphabetic substitution cipher that encodes the message in a sort of binary using 'a's and 'b's, fonts or cases, or in this case, randomized digits where digits 6 and below are 0's and 7 and above are 1's. Each character is stored in 5 bits representing the ASCII.\n\nNote: due to the nature of this cipher, numbers in the secret message will not translate well through the encryption and decryption process.")
        },
        opt if opt.contains("railfence") => {
            String::from("A Railfence cipher is a transposition cipher that shuffles each character according to a number of rails that act as the key.")
        },
        opt if opt.contains("bruteforce") => {
            String::from("This will attempt a bruteforce on a string encoded using one of the available cipher types.")
        },
        opt if opt.contains("score") => {
            String::from("Use this to score a string in terms of how likely it is to be english.")
        },
        opt if opt.contains("polybius") => {
            String::from("A Polybius cipher is a monoalphabetic substitution cipher that shifts values by one row according to a 5x5 alphabetic table.")
        },
        opt if opt.contains("simplesub") => {
            String::from("A simple subsitution cipher is a common monoalphabetic substitution cipher that shifts letters by random values seeded by a given key password.")
        },
        opt if opt.contains("base64") => {
            String::from("Base 64 encodes a string in base 64 (6-bit strings) and mapped to a set of 64 characters. It is not secure, but can be used as a primitive means of obscuring data to the untrained eye.")
        },
        opt if opt.contains("autokey") => {
            String::from("The autokey cipher is polyalphabetic substitution cipher that shifts values according to both the secret key and the plaintext, making the distribution of characters more similar than a vigenere cipher.")
        },
        opt if opt.contains("columnar") => {
            String::from("A Columnar-transpositional cipher is a transpositional cipher that involves transposing laying characters out on a table based on a key then shifting the column order to be based alphabetically on the key. The columns are then listed to get the ciphertext. \n\nNote: A key must have entirely unique characters to function correctly, as the key is alphabetized.")
        },
        _ => {
            String::from("Error: Nothing was selected to retrieve information about!")
        }
    }
}
