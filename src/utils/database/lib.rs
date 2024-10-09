use serde::{de::DeserializeOwned, Deserialize};
use sqlx::{postgres::PgPoolOptions, PgPool};

#[derive(Clone)]
pub struct DatabaseConnection {
    pub pool: PgPool,
}

pub async fn connect(database_url: &str) -> DatabaseConnection {
    DatabaseConnection {
        pool: PgPoolOptions::new()
            .max_connections(4)
            .connect(database_url)
            .await
            .unwrap_or_else(|e| {
                tracing::error!("{:}", e);
                panic!("Error connecting to database {}", database_url)
            }),
    }
}

pub async fn migrate(db_conn: DatabaseConnection) {
    match sqlx::migrate!().run(&db_conn.pool).await {
        Ok(_) => (),
        Err(err) => {
            tracing::error!("{}", err);
            panic!("Failed to run database migrations");
        }
    }
}

pub mod pagination {
    use serde::Deserialize;
    use serde_json::Value;

    #[derive(Deserialize)]
    pub struct Meta {
        pub total: u32,
        pub per_page: u32,
        pub page: u32,
    }

    #[derive(Deserialize)]
    pub struct List<T> {
        pub items: Vec<T>,
    }

    impl<T: serde::de::DeserializeOwned> From<Option<Value>> for List<T> {
        fn from(option: Option<Value>) -> Self {
            match option {
                Some(value) => {
                    let items: Vec<T> = serde_json::from_value(value).unwrap_or_else(|_| vec![]);
                    List { items }
                }
                None => List { items: vec![] },
            }
        }
    }

    #[derive(Deserialize)]
    pub struct Paginated<T> {
        pub items: List<T>,
        pub meta: Meta,
    }
}

#[macro_export]
macro_rules! define_paginated {
    ($name:ident) => {
        #[derive(Debug, Deserialize)]
        pub struct concat_idents!(DatabasePaginated, $name) {
            pub items: List<$name>,
            pub meta: Meta,
        }
    };
}

// impl<T> Into<DatabasePaginated<T>> for Option<serde_json::Value>
// where
//     T: DeserializeOwned,
// {
//     fn into(self) -> DatabasePaginated<T> {
//         match self {
//             Some(json) => {
//                 tracing::info!("About to deserialize: {}", json);
//                 // Use the DeserializeOwned bound for the type T
//                 serde_json::from_value::<DatabasePaginated<T>>(json).unwrap()
//             }
//             None => DatabasePaginated {
//                 data: vec![],
//                 total: 0,
//             },
//         }
//     }
// }
