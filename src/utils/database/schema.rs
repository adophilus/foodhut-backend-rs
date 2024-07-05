diesel::table! {
    users (id) {
        id -> Varchar,
        email -> Varchar,
        phone_number -> Varchar,
        is_verified -> Bool,
        first_name -> Varchar,
        last_name -> Varchar,
        profile_picture_url -> Nullable<Varchar>,
        birthday -> Timestamp,
        referral_code -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    sessions (id) {
        id -> Varchar,
        user_id -> Varchar,
        expires_at -> Timestamp,
    }
}

diesel::table! {
    otps (id) {
        id -> Varchar,
        otp -> Varchar,
        type -> Varchar,
        meta -> Varchar,
        expires_at -> Timestamp,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    kitchens (id) {
        id -> Varchar,
        name -> Varchar,
        type -> Varchar,
        address -> Varchar,
        phone_number -> Varchar,
        opening_time -> Varchar,
        closing_time -> Varchar,
        cover_image_url -> Varchar,
        rating -> Numeric,
        owner_id -> Varchar,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    meals (id) {
        id -> Varchar,
        name -> Varchar,
        description -> Varchar,
        price -> Numeric,
        rating -> Numeric,
        tags -> Json,
        cover_image_url -> Varchar,
        thumbnail_image_url -> Varchar,
        is_available -> Bool,
        kitchen_id -> Varchar,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    carts (id) {
        id -> Varchar,
        items -> Json,
        status -> Varchar,
        user_id -> Varchar,
        created_at -> Timestamp,
        updated_at -> Nullable<Timestamp>,
    }
}
