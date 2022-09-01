use std::fmt::Debug;
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

#[async_trait]
impl<T> TheTrait for T where T: Service<TheTraitRequest, Response=TheTraitResponse> + Send, T::Error: Debug, T::Future: Send {
    async fn bla(&mut self, value1: String, value2: SomeStruct) {
        let response = self.call(TheTraitRequest::Bla { value1, value2 }).await.unwrap();
        match response {
            TheTraitResponse::Bla {} => (),
            _ => panic!("Invalid response variant")
        }
    }

    async fn blub(&mut self) -> SomeStruct {
        let response = self.call(TheTraitRequest::Blub {}).await.unwrap();
        match response {
            TheTraitResponse::Blub(s) => s,
            _ => panic!("Invalid response variant")
        }
    }
}

use tower::Service;

fn use_as_trait(_s: &dyn TheTrait) {}

fn main() {
    let service = TheTraitService::new(TheStruct {});
    use_as_trait(&service);
}
