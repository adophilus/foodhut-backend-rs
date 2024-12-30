ALTER TABLE kitchens
ADD CONSTRAINT fk_kitchens_city_id FOREIGN KEY (city_id) REFERENCES kitchen_cities (id);
