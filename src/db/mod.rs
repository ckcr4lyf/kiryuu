use redis::AsyncCommands;
use crate::byte_functions::types;

#[inline(always)]
pub async fn get_hash_keys_scan(rc: &mut redis::aio::MultiplexedConnection, key: &types::RawVal<22>, count: usize) -> Vec<[u8; 6]> {
    // TBD: what if its Vec<Vec<u8>> in the context of memory allocation
    let mut keys: Vec<[u8; 6]> = Vec::with_capacity(count);
    let mut iter = rc.hscan::<&types::RawVal<22>, ([u8; 6], u8)>(&key).await.expect("fail to scan");

    while let Some((peer, _)) = iter.next_item().await {
        keys.push(peer);

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
    let mut iter = rc.hscan::<&types::RawVal<22>, ([u8; 6], u8)>(&key).await.expect("fail to scan");

    while let Some((peer, _)) = iter.next_item().await {
        keys[pos] = peer;
        pos += 1;

        if pos == count {
            return (keys, pos);
        }
    }

    return (keys, pos);
}

fn example0(f: &dyn Fn() -> [u8; 6]) -> Vec<Vec<u8>> {
    let mut result: Vec<Vec<u8>> = vec![];
    for i in 0..50 {
        let data: [u8; 6] = f();
        result.push(data.to_vec());
    }
    result
}

fn example1(f: &dyn Fn() -> [u8; 6]) -> Vec<Vec<u8>> {
    let mut result: Vec<Vec<u8>> = Vec::with_capacity(50);
    for i in 0..50 {
        let data: [u8; 6] = f();
        result.push(data.to_vec());
    }
    result
}

fn example2(f: &dyn Fn() -> [u8; 6]) -> Vec<[u8; 6]> {
    let mut result: Vec<[u8; 6]> = Vec::with_capacity(50);
    for i in 0..50 {
        let data: [u8; 6] = f();
        result.push(data);
    }
    result
}

fn example3(f: &dyn Fn() -> [u8; 6]) -> [[u8; 6]; 50] {
    let mut result: [[u8; 6]; 50] = [[0; 6]; 50];
    for i in 0..50 {
        let data: [u8; 6] = f();
        result[i] = data;
    }
    result
}