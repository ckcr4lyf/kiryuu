use actix_web::{get, http::{header, StatusCode}, post, web, HttpRequest, HttpResponse};
use serde::Deserialize;

use crate::byte_functions;
use crate::AppState;
use crate::store::PeerPool;

#[get("/test/peer-exists")]
pub async fn test_peer_exists(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> HttpResponse {
    let Some(info_hash) = info_hash_from_request(&req) else {
        return HttpResponse::build(StatusCode::BAD_REQUEST).body("invalid info_hash");
    };

    let pool = match pool_from_request(&req) {
        Ok(pool) => pool,
        Err(resp) => return resp,
    };

    let port = match port_from_request(&req) {
        Ok(port) => port,
        Err(resp) => return resp,
    };

    let peer = match peer_from_request(&req, port) {
        Ok(peer) => peer,
        Err(resp) => return resp,
    };

    if data.store.peer_exists(info_hash, pool, peer) {
        HttpResponse::build(StatusCode::OK).finish()
    } else {
        HttpResponse::build(StatusCode::NOT_FOUND).finish()
    }
}

#[post("/test/seed")]
pub async fn test_seed(
    req: HttpRequest,
    data: web::Data<AppState>,
    body: web::Bytes,
) -> HttpResponse {
    let Some(info_hash) = info_hash_from_request(&req) else {
        return HttpResponse::build(StatusCode::BAD_REQUEST).body("invalid info_hash");
    };

    let pool = match pool_from_request(&req) {
        Ok(pool) => pool,
        Err(resp) => return resp,
    };

    if body.len() % 6 != 0 {
        return HttpResponse::build(StatusCode::BAD_REQUEST).body("body must be a multiple of 6 bytes");
    }

    let peer_count = body.len() / 6;
    let mut peers = Vec::with_capacity(peer_count);
    for chunk in body.chunks_exact(6) {
        let mut peer = [0u8; 6];
        peer.copy_from_slice(chunk);
        peers.push(peer);
    }

    data.store.seed_peers(info_hash, pool, &peers);
    HttpResponse::build(StatusCode::OK)
        .append_header(header::ContentType::plaintext())
        .body(format!("seeded {peer_count} peers\n"))
}

#[derive(Deserialize)]
struct TestQuery {
    info_hash: String,
    pool: String,
    port: Option<u16>,
}

fn info_hash_from_request(req: &HttpRequest) -> Option<[u8; 20]> {
    let parsed = parse_test_query(req)?;
    Some(byte_functions::url_encoded_to_raw_u8(&parsed.info_hash))
}

fn pool_from_request(req: &HttpRequest) -> Result<PeerPool, HttpResponse> {
    let parsed = parse_test_query(req).ok_or_else(|| {
        HttpResponse::build(StatusCode::BAD_REQUEST).body("invalid query")
    })?;

    match parsed.pool.as_str() {
        "s" => Ok(PeerPool::Seeder),
        "l" => Ok(PeerPool::Leecher),
        _ => Err(HttpResponse::build(StatusCode::BAD_REQUEST).body("pool must be s or l")),
    }
}

fn port_from_request(req: &HttpRequest) -> Result<u16, HttpResponse> {
    let parsed = parse_test_query(req).ok_or_else(|| {
        HttpResponse::build(StatusCode::BAD_REQUEST).body("invalid query")
    })?;
    Ok(parsed.port.unwrap_or(4444))
}

fn parse_test_query(req: &HttpRequest) -> Option<TestQuery> {
    let query = req.query_string();
    if query.is_empty() {
        return None;
    }
    serde_qs::from_bytes(query.replace('%', "%25").as_bytes()).ok()
}

fn peer_from_request(req: &HttpRequest, port: u16) -> Result<[u8; 6], HttpResponse> {
    let peer_addr = req.peer_addr().ok_or_else(|| {
        HttpResponse::build(StatusCode::BAD_REQUEST).body("Missing IP")
    })?;

    let std::net::SocketAddr::V4(v4_addr) = peer_addr else {
        return Err(HttpResponse::build(StatusCode::BAD_REQUEST).body("IPv6 not supported"));
    };

    Ok(byte_functions::ip_str_port_u16_to_bytes(v4_addr.ip(), port))
}
