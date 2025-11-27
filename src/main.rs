use std::fs::File;
use std::sync::Arc;

use kasane_logic::{
    encode_id_kv::{Database, EncodeIDKVMultiStore, EncodeIDKVStore},
    encode_id_set::EncodeIDSet,
    id,
    space_time_id::SpaceTimeID,
};
use std::io::Write;

fn main() {
    // Example of EncodeIDSet usage (existing functionality)
    let mut set1 = EncodeIDSet::new();
    let mut set2 = EncodeIDSet::new();

    let id1 = id! {
        z: 6,
        f: [-1, 10],
        x: [2, 10],
        y: [5, 10],
    }
    .unwrap();

    let id2 = id! {
        z: 4,
        f: [-1, 10],
        x: [2, 10],
        y: [5, 10],
    }
    .unwrap();

    let id3 = id! {
        z: 1,
        f: [1, 1],
        x: [1, 1],
        y: [1, 1],
    }
    .unwrap();

    let id4 = id! {
        z: 2,
        f: [2, 2],
        x: [1, 1],
        y: [1, 1],
    }
    .unwrap();

    let id5 = id! {
        z: 1,
        f: [0, 0],
        x: [0, 0],
        y: [0, 0],
    }
    .unwrap();

    let mut file1 = File::create("output1.txt").expect("cannot create file");
    let mut file2 = File::create("output2.txt").expect("cannot create file");
    let mut file3 = File::create("output3.txt").expect("cannot create file");

    id1.to_encode().iter().for_each(|encode_id| {
        set1.insert(encode_id.clone());
    });

    id2.to_encode().iter().for_each(|encode_id| {
        set1.insert(encode_id.clone());
    });

    id3.to_encode().iter().for_each(|encode_id| {
        set1.insert(encode_id.clone());
    });

    id4.to_encode().iter().for_each(|encode_id| {
        set2.insert(encode_id.clone());
    });

    id5.to_encode().iter().for_each(|encode_id| {
        set2.insert(encode_id.clone());
    });

    for ele in set1.iter() {
        writeln!(file1, "{},", ele.decode()).expect("cannot write to file");
    }

    for ele in set2.iter() {
        writeln!(file2, "{},", ele.decode()).expect("cannot write to file");
    }

    let set3 = set1.difference(&set2);

    for ele in set3.iter() {
        writeln!(file3, "{},", ele.decode()).expect("cannot write to file");
    }

    // Example of EncodeIDKVStore usage (new functionality)
    println!("\n=== EncodeIDKVStore Example (Single-value mode) ===");

    // Create a database
    let db = Arc::new(Database::create("example_kv.redb").expect("Failed to create database"));

    // Create a single-value store
    let single_store: EncodeIDKVStore<String> =
        EncodeIDKVStore::new(Arc::clone(&db), "example_single").expect("Failed to create store");

    // Get a sample EncodeID
    let sample_id = id3.to_encode().first().unwrap().clone();

    // Insert a value
    single_store
        .insert(&sample_id, &"Hello from single store!".to_string())
        .expect("Failed to insert");

    // Get the value
    if let Some(value) = single_store.get(&sample_id).expect("Failed to get") {
        println!("Single store value: {}", value);
    }

    // Check store length
    println!(
        "Single store length: {}",
        single_store.len().expect("Failed to get length")
    );

    // Create another map using the same database
    let single_store2: EncodeIDKVStore<String> =
        single_store.create_map("another_single").expect("Failed to create map");

    single_store2
        .insert(&sample_id, &"Value from second map".to_string())
        .expect("Failed to insert");

    println!(
        "Second map value: {:?}",
        single_store2.get(&sample_id).expect("Failed to get")
    );

    // Example of EncodeIDKVMultiStore usage (new functionality)
    println!("\n=== EncodeIDKVMultiStore Example (Multi-value mode) ===");

    // Create a multi-value store
    let multi_store: EncodeIDKVMultiStore<String> =
        EncodeIDKVMultiStore::new(Arc::clone(&db), "example_multi").expect("Failed to create store");

    // Insert multiple values for the same key
    multi_store
        .insert(&sample_id, &"First value".to_string())
        .expect("Failed to insert");
    multi_store
        .insert(&sample_id, &"Second value".to_string())
        .expect("Failed to insert");
    multi_store
        .insert(&sample_id, &"Third value".to_string())
        .expect("Failed to insert");

    // Get all values for the key
    let values = multi_store.get(&sample_id).expect("Failed to get");
    println!("Multi store values for sample_id:");
    for (i, value) in values.iter().enumerate() {
        println!("  {}: {}", i + 1, value);
    }

    // Check store length
    println!(
        "Multi store length (key-value pairs): {}",
        multi_store.len().expect("Failed to get length")
    );

    // Remove a specific value
    multi_store
        .remove(&sample_id, &"Second value".to_string())
        .expect("Failed to remove");

    let values_after_remove = multi_store.get(&sample_id).expect("Failed to get");
    println!("Multi store values after removing 'Second value':");
    for (i, value) in values_after_remove.iter().enumerate() {
        println!("  {}: {}", i + 1, value);
    }

    // Demonstrate thread safety
    println!("\n=== Thread Safety Example ===");

    let store_for_thread = Arc::new(single_store);
    let sample_id_clone = sample_id.clone();

    let handles: Vec<_> = (0..3)
        .map(|i| {
            let store = Arc::clone(&store_for_thread);
            let id = sample_id_clone.clone();
            std::thread::spawn(move || {
                let value = format!("Thread {} value", i);
                store.insert(&id, &value).expect("Failed to insert");
                println!("Thread {} inserted value", i);

                if let Some(read_value) = store.get(&id).expect("Failed to get") {
                    println!("Thread {} read value: {}", i, read_value);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Clean up database file
    std::fs::remove_file("example_kv.redb").ok();

    println!("\nDone!");
}
