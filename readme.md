# Osgiliath

Bridges the Anduin between the towers...

__Disclaimer: The author of this repository is fairly new with Rust (especially macros)
a lot of the implementation may be plain wrong or violate best practices. The idea 
of this repository is to demonstrate that it's possible and eventually gather feedback
from the community.__

This repository contains a __prototype__ macro that generates code allowing to use
`async traits` (via [async_trait crate](https://github.com/dtolnay/async-trait))
as [tower's](https://github.com/tower-rs/tower) `tower::Service` and vice versa.

The goals of this are two fold:
 - easily use of any tower layers (timeout, rate limits, ...) with async traits.
 - prototype external services quickly as part of the application via `tower::Service`
   which eventually can be replaced with clients. 

The idea is to enable the following without having to write any glue code:

```rust
use async_trait::async_trait;
use osgiliath::tower_service;
use tower::Service;

#[tower_service] // includes #[async_trait]
trait TheTrait: Send {
    async fn bla(&mut self, value1: String, value2: usize);
    async fn blub(&mut self) -> f32;
}

struct TheStruct;

#[async_trait]
impl TheTrait for TheStruct {
    async fn bla(&mut self, _value1: String, _value2: usize) {}
    async fn blub(&mut self) -> f32 { return 4.20; }
}

fn use_as_trait(_s: &dyn TheTrait) {}

fn use_as_service(_s: &impl Service<TheTraitRequest>) {}

fn main() {
    // instantiate a wrapper around TheStruct that implements tower::Service
    let service = TheTraitService::new(TheStruct {});
    use_as_service(&service);
    // any Service<TheTraitRequest> also implements the original trait
    use_as_trait(&service);
}
```

## Acknowledgments
This example would not have been possible without the excellent documentation of the
`syn`, `quote`, `async_trait`, and `proc_macro2` crates as well as the 
[macro workshop template repository](https://github.com/dtolnay/proc-macro-workshop).  
