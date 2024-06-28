Cipher tool that encodes and decodes test to various cipher types. 
Currently allows for: 
caesar, vigenere, atbash, affine, baconian, railfence, and ROT13 cipher types. More to be added in the future.

Also allows for brute forcing of all cipher types including vigenere, with a likelihood analysis tool to try to guess the correct output based on how close to english it seems. 

Uses Rust and Tokio to handle asynchronicity and concurrency for the vigenere brute-force.
