use std::error::Error;

use actix_web::web::{BufMut, BytesMut};
use tokio_postgres::types::{accepts, to_sql_checked, IsNull, Type};

// Define a struct to wrap [u8; _] values
// So we can implement redis::ToRedisArgs on them
// directly (i.e. binary redis arg)
#[derive(Debug)]
pub struct RawVal<const T: usize>(pub [u8; T]);


impl<const T: usize> tokio_postgres::types::ToSql for RawVal<T> {
    fn to_sql(&self, _ty: &Type, w: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        w.put_slice(&self.0);
        Ok(IsNull::No)
    }

    accepts!(BYTEA);

    to_sql_checked!();
}

impl<const T: usize> redis::ToRedisArgs for RawVal<T> {
    fn write_redis_args<W>(&self, out: &mut W) where W: ?Sized + redis::RedisWrite {
        out.write_arg(&self.0)
    }
}

impl<const T: usize> std::ops::Index<usize> for RawVal<T> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        return &self.0[index];
    }
}
