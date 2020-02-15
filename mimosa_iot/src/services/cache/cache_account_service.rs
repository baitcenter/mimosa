// use actix_redis::{Command,Error as RedisError,RedisActor, RespValue};
// use actix::Addr;
// use crate::entity::user::{
//     base::UserBase,
//     cache::CacheUser,
// };
// use actix_web::web;

// pub async fn cache_user_base(result:UserBase,redis:&web::Data<Addr<RedisActor>>,)->Result<String,RedisError>{
//     let cache=CacheUser{
//         id:result.id,
//         name:result.user_name,
//         nick_name:result.nick_name,
//         followers:0,
//         following:0,
//         posts:0,
//     };
//     let json = serde_json::to_string(&cache).unwrap();
//     let res = redis.send(Command(resp_array!["HSET","users:$result.id",json])).await? ;
//     match res {
//         Ok(RespValue::SimpleString(s)) if s=="OK"=>Ok(s),
//         Err(err)=>Err(err)
//     }
//    }