use actix_web::{Error, HttpRequest, HttpResponse, Responder};
use anyhow::Result;
use futures::future::{ready, Ready};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::{FromRow, Row, PgPool};
use chrono::{ NaiveDateTime };


#[derive(Serialize, Deserialize)]
pub struct UserRequest {
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub password: String,
    pub mobilephone: String,
    pub isactive: bool,
}


#[derive(Serialize, FromRow, Debug)]
pub struct User {
    pub id: i32,
    pub firstname: Option<String>,
    pub lastname: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub mobilephone: Option<String>,
    pub creationdate: NaiveDateTime,
    pub lastupdate: NaiveDateTime,
    pub isactive: Option<bool>,
}

#[derive(Serialize, FromRow, Debug)]
pub struct UserTable {
    items: Vec<User>,
    total_count: i64
}

#[derive(Deserialize)]
pub struct PaginationRequest {
   pub filter: String,
   pub sortcolumn: String,
   pub sortorder: String,
   pub pagenumber: u32,
   pub pagesize: u32,
}


impl Responder for User {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();

        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}

#[derive(Debug)]
pub struct Respuesta { pub count: Option<i64> }


impl User {
    pub async fn find_all(pool: &PgPool) -> Result<Vec<User>> {
        let mut users = vec![];
        let recs = sqlx::query!(
            r#" SELECT id, firstname, lastname, email, mobilephone, creationdate, lastupdate, isactive FROM public.user ORDER BY id "#
        )
        .fetch_all(pool)
        .await?;

        for rec in recs {
            users.push( User {
                id: rec.id,
                firstname: rec.firstname,
                lastname: rec.lastname,
                email: rec.email,
                password: None,
                mobilephone: rec.mobilephone,
                creationdate: rec.creationdate,
                lastupdate: rec.lastupdate,
                isactive: rec.isactive
            });
        }

        Ok(users)
    }

    pub async fn find_all_with_pagination(pool: &PgPool, filter: &String, sort_column: &String, sort_order: &String) -> Result<UserTable> {
        let mut users = vec![];
        let filter_wildcard = format!("%{}%", &filter);
        let _sc = format!("{}", "firstname");

        let _so = "ASC".to_string();


        

        let recs = sqlx::query!("
        SELECT id,
               firstname,
               lastname,
               email,
               mobilephone,
               creationdate,
               lastupdate,
               isactive
        FROM   public.user
        WHERE  firstname LIKE $1 
        ORDER  BY id", filter_wildcard )
        //.bind(filter_wildcard)
        //.bind(sort_column)
        //.bind(so)
        .fetch_all(pool)
        .await?;

        let rec_count = sqlx::query_as!(Respuesta, 
            r#"SELECT COUNT(*) as count FROM public.user"#)
        .fetch_all(pool)
        .await;

        let count = rec_count.ok().unwrap()[0].count.unwrap();

        for rec in recs {
            users.push( User {
                id: rec.id,
                firstname: rec.firstname,
                lastname: rec.lastname,
                email: rec.email,
                password: None,
                mobilephone: rec.mobilephone,
                creationdate: rec.creationdate,
                lastupdate: rec.lastupdate,
                isactive: rec.isactive
            });
        }

        let user_table = UserTable {
            items: users,
            total_count: count,
        };

        Ok(user_table)
    }

    pub async fn find_by_id(id: i32, pool: &PgPool) -> Result<User> {
        let rec = sqlx::query!(r#" SELECT id, firstname, lastname, email, mobilephone, creationdate, lastupdate, isactive FROM public.user WHERE id = $1 "#,
            id
        )
        .fetch_one(&*pool)
        .await?;

        Ok( User {
            id: rec.id,
            firstname: rec.firstname,
            lastname: rec.lastname,
            email: rec.email,
            password: None,
            mobilephone: rec.mobilephone,
            creationdate: rec.creationdate,
            lastupdate: rec.lastupdate,
            isactive: rec.isactive,
        })
    }

    pub async fn create(user: UserRequest, pool: &PgPool) -> Result<User> {
        let mut tx = pool.begin().await?;
        let user = sqlx::query("INSERT INTO public.user (firstname, lastname, email, password, mobilephone, isactive) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id")
            .bind(&user.firstname)
            .bind(user.lastname)
            .bind(user.email)
            .bind(user.password)
            .bind(user.mobilephone)
            .bind(user.isactive)
            .map(|row: PgRow| {
                User {
                    id: row.get(0),
                    firstname: row.get(1),
                    lastname: row.get(2),
                    email: row.get(3),
                    password: row.get(4),
                    mobilephone: row.get(5),
                    creationdate: row.get(6),
                    lastupdate: row.get(7),
                    isactive: row.get(8),
                }
            })
            .fetch_one(&mut tx)
            .await?;

        tx.commit().await?;
        Ok(user)
    }

    pub async fn update(id: i32, user: UserRequest, pool: &PgPool) -> Result<User> {
        let mut tx = pool.begin().await.unwrap();
        let user = sqlx::query("UPDATE public.user SET firstname = $1, lastname = $2, email = $3, password = $4, mobilephone = $5, isactive = $6 WHERE id = $3 RETURNING id")
            .bind(&user.firstname)
            .bind(user.lastname)
            .bind(user.email)
            .bind(user.password)
            .bind(user.mobilephone)
            .bind(user.isactive)
            .bind(id)
            .map(|row: PgRow| {
                User {
                    id: row.get(0),
                    firstname: row.get(1),
                    lastname: row.get(2),
                    email: row.get(3),
                    password: row.get(4),
                    mobilephone: row.get(5),
                    creationdate: row.get(6),
                    lastupdate: row.get(7),
                    isactive: row.get(8),
                }
            })
            .fetch_one(&mut tx)
            .await?;

        tx.commit().await.unwrap();
        Ok(user)
    }

    pub async fn delete(id: i32, pool: &PgPool) -> Result<u64> {
        let mut tx = pool.begin().await?;
        let _deleted = sqlx::query("DELETE FROM public.user WHERE id = $1")
            .bind(id)
            .execute(&mut tx)
            .await?;

        tx.commit().await?;
        Ok(1)
    }
}
