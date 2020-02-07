
#[derive(Deserialize,Serialize)]
pub struct CacheUser {
   
    pub id: i32,
    pub name: String,
    pub nick_name: String,
    pub followers: i32,
    pub following : i32,
    pub posts:i32,

}