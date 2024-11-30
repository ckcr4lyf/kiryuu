use kiryuu::byte_functions::types;
use redis::AsyncCommands;

pub async fn get_hash_keys_scan(rc: &mut redis::aio::MultiplexedConnection, key: &types::RawVal<22>, count: usize) -> Vec<[u8; 6]> {
    // TBD: what if its Vec<Vec<u8>> in the context of memory allocation
    let mut keys: Vec<[u8; 6]> = Vec::with_capacity(count);
    let mut iter = rc.hscan::<&types::RawVal<22>, [u8; 6]>(&key).await.expect("fail to scan");

    while let Some(element) = iter.next_item().await {
        keys.push(element);

        if keys.len() == count {
            return keys;
        }
    }

    return keys;
}

pub async fn get_hash_keys_scan_stack(rc: &mut redis::aio::MultiplexedConnection, key: &types::RawVal<22>, count: usize) ->  ([[u8; 6]; 50], usize) {
    let mut keys: [[u8; 6]; 50] = [[0; 6]; 50];
    let mut pos: usize = 0;

    let done = 0;
    let mut iter = rc.hscan::<&types::RawVal<22>, [u8; 6]>(&key).await.expect("fail to scan");

    while let Some(element) = iter.next_item().await {
        keys[pos] = element;
        pos += 1;

        if pos == count {
            return (keys, pos);
        }
    }

    return (keys, pos);
}