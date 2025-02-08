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
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Meta {
        pub total: u32,
        pub per_page: u32,
        pub page: u32,
    }

    impl From<Meta> for crate::utils::pagination::PaginatedMeta {
        fn from(m: Meta) -> Self {
            Self {
                total: m.total,
                page: m.page,
                per_page: m.per_page,
            }
        }
    }

    impl From<Option<Value>> for Meta {
        fn from(option: Option<Value>) -> Self {
            match option {
                Some(value) => serde_json::from_value(value).expect("Invalid meta found"),
                None => unreachable!("Invalid meta found"),
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Item<T>(pub Vec<T>);

    impl<T: serde::de::DeserializeOwned> From<Option<Value>> for Item<T> {
        fn from(option: Option<Value>) -> Self {
            match option {
                Some(value) => serde_json::from_value(value).expect("Invalid list found"),
                None => unreachable!("Invalid list found"),
            }
        }
    }
}

#[macro_export]
macro_rules! define_paginated {
    ($name:ident, $type:ty) => {
        #[derive(Debug, Deserialize, Serialize)]
        pub struct $name {
            pub items: crate::utils::database::pagination::Item<$type>,
            pub meta: crate::utils::database::pagination::Meta,
        }

        impl From<$name> for crate::utils::pagination::Paginated<$type> {
            fn from(db_paginated: $name) -> Self {
                Self {
                    items: db_paginated.items.0,
                    meta: db_paginated.meta.into(),
                }
            }
        }
    };
}
