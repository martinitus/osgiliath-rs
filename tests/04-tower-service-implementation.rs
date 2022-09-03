use osgiliath::tower_service;
use async_trait::async_trait;

pub struct SomeStruct {
    value: usize,
}

#[tower_service] // includes #[async_trait]
pub trait TheTrait: Send {
    async fn bla(&mut self, value1: String, value2: SomeStruct);
    async fn blub(&mut self) -> SomeStruct;
    async fn bla_blub(&mut self, u: usize, v: usize) -> ();
}

struct TheStruct;

#[async_trait]
impl TheTrait for TheStruct {
    async fn bla(&mut self, _value1: String, _value2: SomeStruct) {}
    async fn blub(&mut self) -> SomeStruct {
        return SomeStruct { value: 1 };
    }
    async fn bla_blub(&mut self, _u: usize, _v: usize) -> () {
        return ();
    }
}

use tower::Service;

fn use_as_service(_s: &impl Service<TheTraitRequest>) {}

fn main() {
    let service = TheTraitService::new(TheStruct {});
    use_as_service(&service);
}
