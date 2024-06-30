Cipher tool that encodes and decodes test to various cipher types. 
Currently allows for: 
caesar, vigenere, atbash, affine, baconian, railfence, Polybius, and ROT13 cipher types. More to be added in the future.

Also allows for brute forcing of all cipher types including vigenere, with a likelihood analysis tool to try to guess the correct output based on how close to english it seems. 

Uses Rust and Tokio to handle asynchronicity and concurrency for the vigenere brute-force.

Examples below.



Brute forcing a vigenere encoded message

![image](https://github.com/LaineySG/cipher_toy/assets/106799436/3efbe019-87e5-4067-a08f-9ca9da66ab7b)



Elapsed time

![image](https://github.com/LaineySG/cipher_toy/assets/106799436/0f68826f-b2b9-48c8-a246-8d6ac9b0e07d)



Encoding and brute forcing a vigenere encoded message asynchronously

![image](https://github.com/LaineySG/cipher_toy/assets/106799436/e6f87b67-b38b-4f26-a6a4-1e6880db67d0)



Encoding and brute forcing a baconian digitally encoded message

![image](https://github.com/LaineySG/cipher_toy/assets/106799436/0bd947c7-c4f8-4e5d-81cf-90c4f19af38d)



Encoding/decoding affine and railfence ciphers, then bruteforcing a railfence cipher.

![image](https://github.com/LaineySG/cipher_toy/assets/106799436/d2897429-7791-4e8e-addb-24fde92ae467)



Another brute force for an affine encoded message.

![image](https://github.com/LaineySG/cipher_toy/assets/106799436/eb7ca607-aa3e-4426-9429-ef3f387ecfe0)
