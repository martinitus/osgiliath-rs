use osgiliath::tower_service;

#[tower_service]
pub trait Bla: Send {
    async fn bla(&mut self);
}

fn main() {}
