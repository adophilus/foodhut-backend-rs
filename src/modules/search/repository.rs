use crate::define_paginated;
use crate::modules::kitchen::repository::Kitchen;
use crate::modules::meal::repository::Meal;
use crate::utils::pagination::Pagination;
use serde::{Deserialize, Serialize};
use sqlx::PgExecutor;

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum MealOrKitchen {
    Meal(Meal),
    Kitchen(Kitchen),
}

define_paginated!(DatabasePaginatedMealOrKitchen, MealOrKitchen);

#[derive(Deserialize)]
pub struct FindManyMealsAndKitchenFilters {
    pub search: String,
}

pub enum Error {
    UnexpectedError,
}

pub async fn find_many_meals_and_kitchens<'e, E: PgExecutor<'e>>(
    e: E,
    filters: FindManyMealsAndKitchenFilters,
    pagination: Pagination,
) -> Result<DatabasePaginatedMealOrKitchen, Error> {
    sqlx::query_as!(
        DatabasePaginatedMealOrKitchen,
        "
            WITH ranked_results AS (
                SELECT 
                    name,
                    'meal' AS type,
                    SIMILARITY(name, $3) AS rank_score,
                    TO_JSONB(meals) || JSONB_BUILD_OBJECT('kitchen', kitchens) AS item
                FROM
                    meals
                INNER JOIN kitchens
                ON
                     meal.kitchen_id = kitchens.id
                UNION ALL
                SELECT 
                    kitchens.name,
                    'kitchen' AS type,
                    SIMILARITY(kitchens.name, $3) AS rank_score,
                    TO_JSONB(kitchens) || JSONB_BUILD_OBJECT(
                        'city', kitchen_cities
                    ) AS item
                FROM
                    kitchens
                INNER JOIN kitchen_cities
                ON
                     kitchens.city_id = kitchen_cities.id
            ),
            filtered_results AS (
                SELECT
                    *
                FROM
                    ranked_results
                WHERE
                    rank_score > 0.1
                    AND (
                        type = 'kitchen'
                        AND (
                            item->>is_available = TRUE
                            AND item->>is_blocked = FALSE
                            AND item->>is_verified = TRUE
                        )
                        OR (
                            type = 'meal'
                            AND (
                                item->>kitchen->>is_available = TRUE
                                AND item->>kitchen->>is_blocked = FALSE
                                AND item->>kitchen->>is_verified = TRUE
                            )
                        )
                    )
                ORDER BY
                    rank_score DESC,
                    name
            ),
            total_count AS (
                SELECT COUNT(name) AS total_rows FROM filtered_results
            ),
            truncated_results AS (
                SELECT
                    item
                FROM
                    filtered_results
                LIMIT $2
                OFFSET ($1 - 1) * $2
            )
            SELECT
                COALESCE(JSONB_AGG(truncated_results.item), '[]'::JSONB) AS items,
                JSONB_BUILD_OBJECT(
                    'page', $1,
                    'per_page', $2,
                    'total', (SELECT total_rows FROM total_count)
                ) AS meta
            FROM
                truncated_results
        ",
        pagination.page as i32,
        pagination.per_page as i32,
        filters.search
    )
    .fetch_one(e)
    .await
    .map_err(|err| {
        tracing::error!(
            "Error occurred while trying to fetch many meals and kitchens: {}",
            err
        );
        Error::UnexpectedError
    })
}
