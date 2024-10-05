ALTER TABLE order_items
ADD COLUMN kitchen_id VARCHAR;

UPDATE order_items
SET kitchen_id = meals.kitchen_id
FROM meals
WHERE order_items.meal_id = meals.id;

ALTER TABLE order_items
ALTER COLUMN kitchen_id SET NOT NULL;