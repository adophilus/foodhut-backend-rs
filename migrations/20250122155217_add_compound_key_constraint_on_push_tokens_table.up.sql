WITH distinct_tokens AS (
    SELECT DISTINCT ON (token, user_id) id
    FROM push_tokens
    ORDER BY token, user_id, created_at -- Keep the earliest entry for each (token, user_id) pair
)

-- Step 2: Delete duplicates that are not in the distinct_tokens CTE
DELETE FROM push_tokens
WHERE id NOT IN (
    SELECT id FROM distinct_tokens
);

ALTER TABLE push_tokens
ADD CONSTRAINT token_user_id_unique UNIQUE (token, user_id);