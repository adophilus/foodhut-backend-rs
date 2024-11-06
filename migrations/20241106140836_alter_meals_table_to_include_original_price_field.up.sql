ALTER TABLE meals
ADD COLUMN original_price NUMERIC;

UPDATE meals
SET original_price = price;

ALTER TABLE meals
ALTER COLUMN original_price SET NOT NULL;