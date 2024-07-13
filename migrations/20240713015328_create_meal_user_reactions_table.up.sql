CREATE TABLE meal_user_reactions (
  id VARCHAR NOT NULL,
  reaction VARCHAR NOT NULL,
  user_id VARCHAR NOT NULL,
  meal_id VARCHAR NOT NULL,
  created_at TIMESTAMP DEFAULT now() NOT NULL,
  updated_at TIMESTAMP
);
