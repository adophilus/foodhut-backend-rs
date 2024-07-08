#! /usr/bin/env bash

for i in $(seq 1 100);
do
  curl -X POST \
    -H 'Content-Type: application/json' \
    -H 'Authorization: Bearer 01J24YYSCDDR8HD5DHRMZBAJ8Y' \
    --data "{ \"name\": \"$i-John's Cafe\", \"address\": \"No 1, Wetheral Road, Owerri, Imo State.\", \"phone_number\": \"+2348123456789\", \"type\": \"Cuisine\", \"opening_time\": \"08:00\", \"closing_time\": \"20:00\", \"delivery_time\": \"10 - 20 mins\", \"preparation_time\": \"28 - 38 mins\" }" \
    'http://127.0.0.1:8000/api/kitchens'
done
