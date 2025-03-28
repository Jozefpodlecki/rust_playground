
# Generic Associated Types (GATs) in Rust

This project demonstrates the use of **Generic Associated Types (GATs)** in Rust, particularly in the context of a **data provider trait** that can return different data representations depending on the borrow lifetime.

## Overview

The code defines a `DataProvider` trait with a **GAT (`Data<'a>`)** to allow borrowing from the struct dynamically based on lifetimes.

The `OwnedProvider` struct implements this trait, returning a borrowed slice of `Payload`.
