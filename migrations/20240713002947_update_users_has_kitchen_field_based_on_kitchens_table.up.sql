UPDATE users SET
  has_kitchen = CASE
    WHEN EXISTS (
      SELECT 1
      FROM kitchens k
      WHERE k.owner_id = users.id
    )
    THEN true
    ELSE false
  END;
