# gleam_finder

This crate contains tools you can use to get gleam giveaways links.

You can search google for every web page referring gleam.io in the last hour with google::search().
After you got these links, you can load the pages and parse the description to get gleam.io links with youtube::resolve().
You can parse a gleam.io page with the Giveaway struct.

## Examples

```rust
use gleam_finder::*;

for page in 0..4 {
    for link in google::search(page).unwrap() {
        println!("resolving {}", link);
        for gleam_link in intermediary::resolve(&link).unwrap() {
            println!("gleam link found: {}", gleam_link);
        }
    }
}
```

License: MIT
