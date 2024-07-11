**To run the ciphertool:**
Download the newest release version and run the exe, or clone the git and run 'cargo build --release' or use your IDE's debug feature to build the project from the rust files. If you build it, make sure to download rockyou.txt and place it in src/data or bruteforcing will throw an error that the file could not be found. rockyou.txt can be found here: 
https://github.com/brannondorsey/naive-hashcat/releases/download/data/rockyou.txt

rockyou.txt is included in the release zip.

Cipher tool that encodes and decodes test to various cipher types. 
Currently allows for: 
caesar, vigenere, atbash, affine, baconian, railfence, Polybius, Autokey, columnar transposition, and ROT13 cipher types. More to be added in the future.

Also allows for brute forcing of many cipher types including vigenere, with a likelihood analysis tool to try to guess the correct output based on how close to english it seems. 

Uses Rust and Tokio to handle asynchronicity and concurrency for the vigenere brute-force.

Examples below.

![image](https://github.com/LaineySG/cipher_toy/assets/106799436/02924af8-f9e2-4f0b-8521-a1f8314e2ec2)

![image](https://github.com/LaineySG/cipher_toy/assets/106799436/e02cf2f9-e190-4e88-9837-477f72aa5dbf)

![image](https://github.com/LaineySG/cipher_toy/assets/106799436/2d7d7389-fed3-4f51-aa5c-59df4b47ba13)

![image](https://github.com/LaineySG/cipher_toy/assets/106799436/29498ff7-dbd9-4342-a502-47b0ae13b20d)

![image](https://github.com/LaineySG/cipher_toy/assets/106799436/8c1163b7-7dd5-4fc3-aa46-e758f51eadf6)
