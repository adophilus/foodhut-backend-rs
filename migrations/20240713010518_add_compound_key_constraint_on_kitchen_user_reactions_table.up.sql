ALTER TABLE kitchen_user_reactions
ADD CONSTRAINT user_id_kitchen_id_unique UNIQUE (user_id, kitchen_id);
