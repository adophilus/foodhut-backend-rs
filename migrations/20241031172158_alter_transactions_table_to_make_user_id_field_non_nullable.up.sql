UPDATE transactions
    SET user_id = wallets.owner_id
FROM wallets
WHERE
    transactions.wallet_id = wallets.id
    AND transactions.user_id IS NULL;

ALTER TABLE transactions
ALTER COLUMN user_id SET NOT NULL;