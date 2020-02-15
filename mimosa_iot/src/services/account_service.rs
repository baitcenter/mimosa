use futures::future::join_all;
use actix_redis::Error;
use bcrypt::{hash, DEFAULT_COST};

use crate::{
    config::db::{Connection,Pool},
    constants,
    error::ServiceError,
    entity::user:: {
        auth::{UserAuth, LoginDTO,UserDTO},
        cache::CacheUser,
        base::{UserBase,UserBaseDto},
        token::UserToken,
    },
    utils::token_utils,
};
use actix::prelude::*;
use actix_redis::{Command, Error as AWError, RedisActor, RespValue};
use actix_web::{
    // client::Client,
    http::{
        StatusCode,
        header::HeaderValue,
    },
    web,
};

use uuid::Uuid;

//注册
#[derive(Serialize,Deserialize)]
pub struct ReqRegist {
    pub identity_type:i32,
    pub identifier:String,
    pub certificate:String,
}

#[derive(Serialize, Deserialize)]
pub struct RespToken {
    pub token:String,
    pub token_type:String,
}

// pub fn check_phone_exist(phone:String,)
pub async fn signup(dto: ReqRegist,pool: &web::Data<Pool>,redis:&web::Data<Addr<RedisActor>>,) -> Result<String, ServiceError>{
    let conn = &pool.get().unwrap();
    if UserAuth::find_user_by_identifier(&dto.identifier,conn).is_err(){
        // 创建用户信息表
        // let baseDto = UserBaseDto{

        // };
        let hashed_pwd = hash(&dto.certificate, DEFAULT_COST).unwrap();
        let base = UserBaseDto{
            register_source:1,
            user_role:2,
            user_name:&dto.identifier,
            mobile:&dto.identifier,
            mobile_bind_time:Some(chrono::Utc::now().naive_utc()),

        };
        match UserBase::insert(base,conn){
            Ok(result)=>{
                let dto = UserDTO{
                    uid:result.id,
                    identity_type:dto.identity_type,
                    identifier:&dto.identifier,
                    certificate:&hashed_pwd,
                };
                match UserAuth::insert(dto, conn) {
                        Ok(message) => {
                            let cache=CacheUser{
                                id:result.id,
                                name:result.user_name,
                                nick_name:result.nick_name,
                                followers:0,
                                following:0,
                                posts:0,
                            };
                            let json = serde_json::to_string(&cache).unwrap();
                            let cache =redis.send(Command(resp_array!["HSET","users:$result.id",json]));

                            let res: Vec<Result<RespValue, AWError>> =
                                join_all(vec![cache].into_iter())
                                    .await
                                    .into_iter()
                                    .map(|item| {
                                        item.map_err(AWError::from)
                                            .and_then(|res| res.map_err(AWError::from))
                                    })
                                    .collect();

                            // successful operations return "OK", so confirm that all returned as so
                            if !res.iter().all(|res| match res {
                                Ok(RespValue::SimpleString(x)) if x == "OK" => true,
                                _ => false,
                            }) {
                                Ok(message)
                            } else {
                                Err(ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, constants::MESSAGE_INTERNAL_SERVER_ERROR.to_string()))
                             }
                           
                            
                            
                        },
                        Err(message) => Err(ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, message))
                    }
            },
            Err(message) => Err(ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, message))
        }
        
    }else{
        Err(ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, format!("User '{}' is already registered", &dto.identifier)))
    }
}

pub fn login(login: LoginDTO, pool: &web::Data<Pool>,redis:&web::Data<Addr<RedisActor>>,) -> Result<RespToken, ServiceError>{
    match UserAuth::login(login, &pool.get().unwrap()) {
        Some(logged_user) => {
            match serde_json::from_value(json!({ "token": UserToken::generate_token(logged_user), "token_type": "bearer" })) {
                Ok(token_res) => Ok(token_res),
                Err(_) => Err(ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, constants::MESSAGE_INTERNAL_SERVER_ERROR.to_string()))
            }
        },
        None => Err(ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, constants::MESSAGE_LOGIN_FAILED.to_string()))
    }
}



pub fn logout(authen_header: &HeaderValue, pool: &web::Data<Pool>) -> Result<(), ServiceError>{
    if let Ok(authen_str) = authen_header.to_str() {
        if authen_str.starts_with("bearer") {
            let token = authen_str[6..authen_str.len()].trim();
            if let Ok(token_data) = token_utils::decode_token(token.to_string()) {
                if let Ok(iden) = token_utils::verify_token(&token_data, pool) {
                    if let Ok(user) = UserAuth::find_user_by_identifier(&iden, &pool.get().unwrap()) {
                        UserAuth::logout(user.id, &pool.get().unwrap());
                        return Ok(());
                    }
                }
            }
        }
    }

    Err(ServiceError::new(StatusCode::INTERNAL_SERVER_ERROR, constants::MESSAGE_PROCESS_TOKEN_ERROR.to_string()))
}
//TODO user-info->hash;home-timeline->redis zset;status->hash;followers->zset;following->zset;
