# Introduction

This is just a fun little project I made to understand how concurrency and synchronization works in Rust. 
Each elevator runs in its own thread and communicates with a central controller through channels. 
It also uses a scoring system to decide which elevator should handle new requests.

Usage:
1. Clone the repository
2. Run the project with `cargo run`

The program will start with three elevator threads and a controller. You can modify and add more elevators or change the type of requests being made in the `main` function if you wish.
