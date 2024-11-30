ALTER TABLE meal_user_reactions
ADD CONSTRAINT user_id_meal_id_unique UNIQUE (user_id, meal_id);
