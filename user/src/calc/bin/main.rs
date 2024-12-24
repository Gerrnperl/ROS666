#![no_std]
#![no_main]

use alloc::vec::Vec;
use lib::println;
extern crate alloc;

#[unsafe(no_mangle)]
pub fn main(_argc: usize, argv: &[&str]) {
    let result = calc(argv.join(" ").as_str());
    println!("{}", result);
}

pub fn calc(str: &str) -> i32 {
    let tokens = split(str);

    let postfix = to_postfix(tokens);

    let mut stack = Vec::new();

    for token in postfix {
        if is_number(token) {
            stack.push(token.parse::<i32>().unwrap());
        } else {
            let b = stack.pop().unwrap();
            let a = stack.pop().unwrap();
            let res = match token {
                "+" => a + b,
                "-" => a - b,
                "*" => a * b,
                "/" => a / b,
                _ => 0,
            };
            stack.push(res);
        }
    }

    stack.pop().unwrap()
}

pub fn to_postfix(tokens: Vec<&str>) -> Vec<&str> {
    let mut res = Vec::new();
    let mut op_stack = Vec::new();
    for token in tokens {
        if is_number(token) {
            res.push(token);
        } else if token == "(" {
            op_stack.push(token);
        } else if token == ")" {
            while let Some(op) = op_stack.pop() {
                if op == "(" {
                    break;
                }
                res.push(op);
            }
        } else {
            while let Some(op) = op_stack.last() {
                if *op == "(" {
                    break;
                }
                if priority(op) >= priority(token) {
                    res.push(op_stack.pop().unwrap());
                } else {
                    break;
                }
            }
            op_stack.push(token);
        }
    }
    while let Some(op) = op_stack.pop() {
        res.push(op);
    }
    res
}

pub fn is_number(str: &str) -> bool {
    str.chars().all(|c| c.is_digit(10))
}

pub fn priority(op: &str) -> i32 {
    match op {
        "+" | "-" => 1,
        "*" | "/" => 2,
        _ => 0,
    }
}

pub fn split(str: &str) -> Vec<&str> {
    let mut res = Vec::new();
    let mut last = 0;
    for (i, c) in str.char_indices() {
        if c.is_whitespace() {
            if last < i {
                res.push(&str[last..i]);
            }
            last = i + 1;
        } else if c.is_ascii_punctuation() {
            if last < i {
                res.push(&str[last..i]);
            }
            res.push(&str[i..i + 1]);
            last = i + 1;
        }
    }
    if last < str.len() {
        res.push(&str[last..]);
    }
    res
}
