use actix_web::web;
use sqlx::PgPool;
use uuid::Uuid;

use crate::model::ConnectionTuple;



pub async fn insert_connection_db( 
    room_uuid: Uuid,
    user_uuid: Uuid,
    connection: web::Data<PgPool>,
    ) -> Result<(), sqlx::Error> {
      sqlx::query!(r#"INSERT INTO connections (user_id, room_id, is_admin) 
            VALUES ($1, $2, $3)"#, user_uuid,room_uuid, false)
        .execute(connection.get_ref())
        .await?;
    Ok(())  
}

pub async fn get_connection_by_room_and_user(
    room_uuid: Uuid,
    user_uuid: Uuid,
    connection: web::Data<PgPool>,
) -> Result<ConnectionTuple, sqlx::Error> {
    let conn_result =  sqlx::query_as!(ConnectionTuple,r#"SELECT * from Connections WHERE room_id = $1 AND user_id = $2"#, room_uuid, user_uuid)
        .fetch_one(connection.get_ref())
        .await?;
    Ok(conn_result)
}

pub async fn delete_room_connections_close_room(
    room_uuid: Uuid,
    connection: web::Data<PgPool>,
)-> Result<(), sqlx::Error>{
    let mut tx = connection.get_ref().begin().await?;

    sqlx::query!(r#"UPDATE AvailableRooms SET is_open=false WHERE room_id = $1"#, room_uuid)
        .execute(&mut tx)
        .await?;
    sqlx::query!(r#"DELETE FROM Connections WHERE room_id = $1"#, room_uuid)
        .execute(&mut tx)
        .await?;
    
    tx.commit().await
}