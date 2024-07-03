use std::io;
use std::env;
use eframe::glow::Context;
use futures::executor::block_on;
use futures::join;
mod ciphers;
use eframe::egui;
use futures::TryFutureExt;
use itertools::Format;
use rand_seeder::rand_core::block;
use tokio::sync::watch;
use tokio::sync::oneshot;
use core::fmt;

fn main() -> Result<(),eframe::Error> {
    eframe::run_native("Cipher Toy", eframe::NativeOptions::default(), Box::new(|cc| Box::new(MainWindow::new(cc))))
}
struct MainWindow {
    message_input:String,
    int_a:i32,
    int_b:i32,
    float_percent:f64,
    key_input:String,
    selected_action:SelectedActionEnum,
    encrypt_or_decrypt:EncOrDec, //True will be encrypt
    result:String
}

#[derive(Debug, PartialEq)]
enum SelectedActionEnum {
    Caesar,Vigenere,Atbash,Affine,Baconian,Polybius,SimpleSub,RailFence,Rot13,Bruteforce, BruteforceVigenere, Score,Unknown,Autokey,Columnar
}
impl fmt::Display for SelectedActionEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
#[derive(Debug, PartialEq)]
enum EncOrDec {
    Encrypt, Decrypt
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
            key_input:String::new(),
            selected_action: SelectedActionEnum::Caesar,
            encrypt_or_decrypt: EncOrDec::Encrypt,
            result: String::new()
        }
    }
}

impl eframe::App for MainWindow {
   fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let Self {message_input,selected_action,encrypt_or_decrypt,result,key_input,int_a,int_b,float_percent} = self;
            

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
                    ui.selectable_value(selected_action, SelectedActionEnum::Columnar, "Columnar Transpositional Cipher");
                    ui.selectable_value(selected_action, SelectedActionEnum::Bruteforce, "Bruteforce");
                    ui.selectable_value(selected_action, SelectedActionEnum::BruteforceVigenere, "Bruteforce Vigenere");
                    ui.selectable_value(selected_action, SelectedActionEnum::Score, "Score String");
                });
                
            ui.separator();
            match selected_action.to_string().to_lowercase() {
                x if x.contains("simplesub") || (x.contains("vigenere") && !x.contains("bruteforce")) || x.contains("autokey") || x.contains("column") => {
                    ui.label("Secret Key");
                    ui.text_edit_singleline(key_input);
                    ui.separator();
                }
                x if x.contains("affine") => {
                    ui.label("Secret Key a"); //a,b : i32
                    ui.add(
                        egui::DragValue::new(int_a).clamp_range(1..=1000)
                    );
                    ui.label("Secret Key b"); //a,b : i32
                    ui.add(
                        egui::DragValue::new(int_b).clamp_range(1..=1000)
                    );
                    *key_input = format!("{},{}",int_a.to_string(),int_b.to_string());
                    ui.separator();
                }
                x if x.contains("caesar") || x.contains("railfence") => {
                    ui.label("Secret Key"); //a: i32ui.add(i32)
                    ui.add(
                        egui::DragValue::new(int_a).clamp_range(1..=1000)
                    );
                    *key_input = int_a.to_string();
                    ui.separator();
                }
                x if x.contains("bruteforce") && !x.contains("vigenere") => {
                    if !key_input.contains("unk") && !key_input.contains("cae") && !key_input.contains("vig")
                    && !key_input.contains("atb") && !key_input.contains("aff") && !key_input.contains("bac")
                    && !key_input.contains("vig") && !key_input.contains("rot") && !key_input.contains("rail")
                    && !key_input.contains("pol") && !key_input.contains("sub") {*key_input = "unknown".to_string()}

                    egui::ComboBox::from_label("Cipher type (if known)")
                    .selected_text(format!("{:?}", key_input))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(key_input, SelectedActionEnum::Unknown.to_string().to_lowercase(), "Unknown Cipher");
                        ui.selectable_value(key_input, SelectedActionEnum::Caesar.to_string().to_lowercase(), "Caesar Cipher");
                        ui.selectable_value(key_input, SelectedActionEnum::Vigenere.to_string().to_lowercase(), "Vigenere Cipher");
                        ui.selectable_value(key_input, SelectedActionEnum::Atbash.to_string().to_lowercase(), "Atbash Cipher");
                        ui.selectable_value(key_input, SelectedActionEnum::Affine.to_string().to_lowercase(), "Affine Cipher");
                        ui.selectable_value(key_input, SelectedActionEnum::Baconian.to_string().to_lowercase(), "Baconian Cipher");
                        ui.selectable_value(key_input, SelectedActionEnum::Polybius.to_string().to_lowercase(), "Polybius Cipher");
                        ui.selectable_value(key_input, SelectedActionEnum::SimpleSub.to_string().to_lowercase(), "Simple Substitution Cipher");
                        ui.selectable_value(key_input, SelectedActionEnum::RailFence.to_string().to_lowercase(), "Railfence Cipher");
                        ui.selectable_value(key_input, SelectedActionEnum::Rot13.to_string().to_lowercase(), "ROT13 Cipher");
                        ui.selectable_value(key_input, SelectedActionEnum::Autokey.to_string().to_lowercase(), "Autokey Cipher");
                    });
                    ui.separator();
                }
                x if x.contains("bruteforce") && x.contains("vigenere") => {
                    ui.label("% of words to check"); //a: i32ui.add(i32)
                    ui.add(
                        egui::DragValue::new(float_percent).clamp_range(1.0..=100.0)
                    );
                    ui.separator();
                }
                _ => {}
            }
            ui.horizontal(|ui| {
                ui.radio_value(encrypt_or_decrypt, EncOrDec::Encrypt, "Encrypt");
                ui.radio_value(encrypt_or_decrypt, EncOrDec::Decrypt, "Decrypt");
            });
            ui.separator();

            if ui.button("Start").clicked() {
                *result = "Working...".to_string();
                *result = run_operations(message_input.to_string(), selected_action.to_string(),
                key_input.to_string(), encrypt_or_decrypt.to_string());
               
            }
            
            let result_description = match encrypt_or_decrypt.to_string().to_lowercase() {
                x if x.contains("enc") => "ciphertext",
                x if x.contains("dec") => "plaintext",
                _=> "output",
            };

            egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(false)
            .min_height(400.0)
            .default_height(400.0)
            .show_inside(ui, |ui| { egui::ScrollArea::vertical().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(format!("Resulting {} is: \t",result_description));
                    ui.label(format!("{result}")).highlight();
                });
            });
            });
        });
    }
}

///main operation running logic
#[tokio::main]
async fn run_operations(message_input:String,selected_action:String,secret_key:String,mut encrypt_or_decrypt:String) -> String {
    encrypt_or_decrypt = encrypt_or_decrypt.to_lowercase();
    match selected_action.to_lowercase() {
        opt if opt.contains("caesar") => {
            if secret_key.trim().to_lowercase().parse::<i32>().is_ok() { 
                let shift_key = secret_key.trim().to_lowercase().parse::<i32>().unwrap(); //Try to get shift key as integer
                let result = ciphers::caesar_cipher(&message_input,shift_key,&encrypt_or_decrypt);
                result
            } else {
                String::from("Please enter the secret key as a positive integer.")
            }
        },
        opt if opt.contains("vigenere") && !opt.contains("bruteforce") => {
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
                    let result = String::from("Error: For the affine cipher, enter the secret key as a comma separated list: a,b");
                    result
                }
            } else {String::from("Error: For the affine cipher, enter the secret key as a comma separated list: a,b")}
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
                String::from("Error: For the railfence cipher, the secret key must be an integer")
            }
        },
        opt if opt.contains("autokey") => {
            let result = ciphers::autokey_cipher(&message_input, &secret_key, &encrypt_or_decrypt);
            result
        },
        opt if opt.contains("bruteforce") && !opt.contains("vigenere") => {
            let result = ciphers::bruteforce(&message_input, "unknown");
            result
        },
        opt if opt.contains("score") => {
            let mut word_list: Vec<String> = vec![];
            if let Ok(lines) = ciphers::read_lines("src/data/1000_most_common.txt") {
                // Consumes the iterator, returns an (Optional) String
                for line in lines.flatten() {
                    word_list.push(line);
                }
                let result = ciphers::score_string(&message_input, &word_list);
                result.to_string()
            } else {String::from("Error: Word list directory not found!")}
        },
        opt if opt.contains("bruteforce") && opt.contains("vigenere") => {
            if secret_key.parse::<i32>().is_ok() {
                let keyasf64 = secret_key.trim().to_lowercase().parse::<i32>().unwrap();
                let bfl = (keyasf64 as f64 / 100.0 * 14344392.0) as i32; //14344392 is the number of passwords in the bruteforce list
                let result = ciphers::bruteforce_vigenere(&message_input, bfl).await;
                if result.is_ok() {
                    result.unwrap()
                } else {
                    String::from("Undefined error.")
                }
            } else {return String::from("Error: Could not parse word count percentage as float!")}
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
        opt if opt.contains("column") => {
            //let result = ciphers::col_trans_cipher();
            String::from("Not added yet")
        }
        _ => {
            String::from("Nothing selected!")
        }
    }
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
            String::from("A baconian cipher is a monoalphabetic substitution cipher that encodes the message in a sort of binary using 'a's and 'b's, fonts or cases, or in this case, randomized digits where digits 6 and below are 0's and 7 and above are 1's. Each character is stored in 5 bits representing the ASCII.")
        },
        opt if opt.contains("railfence") => {
            String::from("A Railfence cipher is a transposition cipher that shuffles each character according to a number of rails that act as the key.")
        },
        opt if opt.contains("bruteforce") && !opt.contains("vigenere") => {
            String::from("This will attempt a bruteforce on a string encoded using one of the available cipher types. Choose vigenere bruteforce to attempt vigenere.")
        },
        opt if opt.contains("score") => {
            String::from("Use this to score a string in terms of how likely it is to be english.")
        },
        opt if opt.contains("bruteforce") && opt.contains("vigenere") => {
            String::from("This will attempt a bruteforce on a string encoded with vigenere. Note that vigenere will take a long time, and may not be possible given a secure enough key.")
        },
        opt if opt.contains("polybius") => {
            String::from("A Polybius cipher is a monoalphabetic substitution cipher that shifts values by one row according to a 5x5 alphabetic table.")
        },
        opt if opt.contains("simplesub") => {
            String::from("A simple subsitution cipher is a common monoalphabetic substitution cipher that shifts letters by random values seeded by a given key password.")
        },
        opt if opt.contains("autokey") => {
            String::from("The autokey cipher is polyalphabetic substitution cipher that shifts values according to both the secret key and the plaintext, making the distribution of characters more similar than a vigenere cipher.")
        },
        opt if opt.contains("columnar") => {
            String::from("A Columnar-transpositional cipher is a transpositional cipher that involves transposing laying characters out on a table based on a key then shifting the column order to be based alphabetically on the key. The columns are then listed to get the ciphertext.")
        },
        _ => {
            String::from("Nothing selected!")
        }
    }
}
