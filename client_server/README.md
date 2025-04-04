# Rust Borrowing and Lifetime Management with `Resolver` and `Service`

This project demonstrates **Rust’s borrowing and lifetime management** using **HashMap**, **lifetimes (`'a`)**, and **struct composition**.

It features a `Resolver` that manages items with references, and a `Service` that interacts with it.

## Overview

The code defines:
- An **`Item<'a>`** struct that holds a reference to a string.
- A **`Resolver<'a>`** struct that manages `Item<'a>` instances in a `HashMap`.
- A **`Service<'a>`** struct that interacts with the `Resolver`.

## Reading

- [03-lifetime-syntax](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html)