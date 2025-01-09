INSERT INTO wallets (id, balance, metadata, is_kitchen_wallet, owner_id, created_at)
SELECT
    gen_random_uuid()::TEXT AS id,
    0 AS balance,
    '{}' AS metadata,
    TRUE AS is_kitchen_wallet,
    kitchens.owner_id AS owner_id,
    NOW() AS created_at
FROM
    kitchens
