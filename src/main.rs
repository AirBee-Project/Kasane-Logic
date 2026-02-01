use kasane_logic::{SingleId, TableOnMemory};

fn main() {
    futures::executor::block_on(async {
        let mut table = TableOnMemory::default();
        let id = SingleId::new(3, 4, 5, 6).unwrap();
        table.insert(&id, &"neko".to_string()).await;
        table.insert(&id, &"inu".to_string()).await;

        println!("{}", table.to_set());
    });
}
