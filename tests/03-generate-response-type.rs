use osgiliath::tower_service;
use async_trait::async_trait;


pub struct SomeStruct {
    value: usize,
}

#[tower_service] // includes #[async_trait]
pub trait TheTrait : Send {
    async fn bla(&mut self, value1: String, value2: SomeStruct);
    async fn blub(&mut self) -> SomeStruct;
}

struct TheStruct;

#[async_trait]
impl TheTrait for TheStruct {
    async fn bla(&mut self, _value1: String, _value2: SomeStruct) {}
    async fn blub(&mut self) -> SomeStruct {
        return SomeStruct { value: 1 };
    }
}

fn main() {
    let _ = TheTraitResponse::Bla;
    let _ = TheTraitResponse::Blub(SomeStruct { value: 1 });
}
