use osgiliath::tower_service;
use async_trait::async_trait;


struct SomeStruct {
    value: usize,
}

#[tower_service] // includes #[async_trait]
trait TheTrait: Send {
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
    let _ = TheTraitRequest::Bla { value1: "hello".to_string(), value2: SomeStruct { value: 1 } };
    let _ = TheTraitRequest::Blub {};
}
