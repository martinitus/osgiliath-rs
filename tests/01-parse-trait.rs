use osgiliath::tower_service;

#[tower_service]
trait Bla: Send {
    async fn bla(&mut self);
}

fn main() {}
