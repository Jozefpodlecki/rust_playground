
```
cargo install sqlx-cli
```

```
$env:DATABASE_URL="sqlite:todos.db"
sqlx db create
```

```
sqlx migrate add <name of migration>
```

## Reading
- [rust-sqlx-basics-with-sqlite](https://tms-dev-blog.com/rust-sqlx-basics-with-sqlite/)