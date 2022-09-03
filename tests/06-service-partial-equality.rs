use osgiliath::tower_service;
use async_trait::async_trait;

#[tower_service] // includes #[async_trait]
pub trait TheTrait: Send {
    async fn bla(&mut self, value1: String, value2: usize);
}

struct TheStruct;

#[async_trait]
impl TheTrait for TheStruct {
    async fn bla(&mut self, _value1: String, _value2: usize) {}
}


fn main() {
    let service1 = TheTraitService::new(TheStruct {});
    let service2 = service1.clone();
    service1.eq(&service2);
}
