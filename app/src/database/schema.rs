diesel::table! {
    accounts (id) {
        id -> Integer,
        name -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    addresses (id) {
        id -> Integer,
        label -> Nullable<Text>,
        address_line1 -> Text,
        address_line2 -> Nullable<Text>,
        city -> Text,
        region -> Nullable<Text>,
        country -> Text,
        postal_code -> Text,
        phone -> Nullable<Text>,
        is_default -> Bool,
        address_type -> Text,
        customer_id -> Nullable<Integer>,
        brand_id -> Nullable<Integer>,
        order_id -> Nullable<Integer>,
        account_id -> Nullable<Integer>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    audit_logs (id) {
        id -> Integer,
        entity -> Text,
        entity_id -> Text,
        action -> Text,
        changes -> Nullable<Text>,
        user_id -> Nullable<Integer>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    brands (id) {
        id -> Integer,
        name -> Text,
        slug -> Text,
        description -> Nullable<Text>,
        website -> Nullable<Text>,
        is_visible -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
        account_id -> Nullable<Integer>,
    }
}

diesel::table! {
    categories (id) {
        id -> Integer,
        name -> Text,
        slug -> Text,
        description -> Nullable<Text>,
        is_visible -> Bool,
        parent_id -> Nullable<Integer>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
        account_id -> Nullable<Integer>,
    }
}

diesel::table! {
    clockwork_queries (id) {
        id -> Integer,
        request_id -> Text,
        sql -> Text,
        bindings -> Nullable<Text>,
        duration -> Double,
        query_type -> Text,
    }
}

diesel::table! {
    clockwork_requests (id) {
        id -> Text,
        version -> Integer,
        request_type -> Text,
        time -> Double,
        method -> Text,
        url -> Text,
        uri -> Text,
        headers -> Nullable<Text>,
        get_data -> Nullable<Text>,
        post_data -> Nullable<Text>,
        cookies -> Nullable<Text>,
        response_status -> Integer,
        response_duration -> Double,
        memory_usage -> BigInt,
        queries_count -> Integer,
        queries_duration -> Double,
        slow_queries -> Integer,
        middleware -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    contacts (id) {
        id -> Integer,
        first_name -> Text,
        last_name -> Text,
        email -> Nullable<Text>,
        phone -> Nullable<Text>,
        address -> Nullable<Text>,
        city -> Nullable<Text>,
        region -> Nullable<Text>,
        country -> Nullable<Text>,
        postal_code -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
        account_id -> Nullable<Integer>,
        organization_id -> Nullable<Integer>,
    }
}

diesel::table! {
    customers (id) {
        id -> Integer,
        name -> Text,
        email -> Nullable<Text>,
        phone -> Nullable<Text>,
        account_id -> Nullable<Integer>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    image_jobs (id) {
        id -> Integer,
        disk -> Text,
        filename -> Text,
        original_filename -> Text,
        options -> Text,
        status -> Text,
        processed_at -> Nullable<Timestamp>,
        accessed_at -> Nullable<Timestamp>,
        access_count -> Integer,
        error_message -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    media (id) {
        id -> Integer,
        filename -> Text,
        original_filename -> Text,
        file_path -> Text,
        mime_type -> Text,
        file_size -> BigInt,
        width -> Nullable<Integer>,
        height -> Nullable<Integer>,
        metadata -> Nullable<Text>,
        title -> Nullable<Text>,
        alt_text -> Nullable<Text>,
        caption -> Nullable<Text>,
        is_public -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        uploaded_by -> Nullable<Integer>,
    }
}

diesel::table! {
    order_items (id) {
        id -> Integer,
        order_id -> Integer,
        product_id -> Integer,
        quantity -> Integer,
        unit_price -> Integer,
        total -> Integer,
    }
}

diesel::table! {
    orders (id) {
        id -> Integer,
        number -> Text,
        status -> Text,
        currency -> Text,
        subtotal -> Integer,
        tax -> Integer,
        shipping_cost -> Integer,
        total -> Integer,
        notes -> Nullable<Text>,
        customer_id -> Nullable<Integer>,
        shipping_address_id -> Nullable<Integer>,
        billing_address_id -> Nullable<Integer>,
        account_id -> Nullable<Integer>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    organizations (id) {
        id -> Integer,
        name -> Text,
        email -> Nullable<Text>,
        phone -> Nullable<Text>,
        address -> Nullable<Text>,
        city -> Nullable<Text>,
        region -> Nullable<Text>,
        country -> Nullable<Text>,
        postal_code -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
        account_id -> Nullable<Integer>,
    }
}

diesel::table! {
    product_categories (product_id, category_id) {
        product_id -> Integer,
        category_id -> Integer,
    }
}

diesel::table! {
    products (id) {
        id -> Integer,
        name -> Text,
        slug -> Text,
        sku -> Nullable<Text>,
        barcode -> Nullable<Text>,
        description -> Nullable<Text>,
        price -> Integer,
        old_price -> Nullable<Integer>,
        cost -> Nullable<Integer>,
        quantity -> Integer,
        security_stock -> Integer,
        stock_status -> Text,
        backorder -> Bool,
        requires_shipping -> Bool,
        published_at -> Nullable<Date>,
        is_visible -> Bool,
        is_featured -> Bool,
        image -> Nullable<Text>,
        brand_id -> Nullable<Integer>,
        account_id -> Nullable<Integer>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        email -> Text,
        first_name -> Text,
        last_name -> Text,
        password -> Text,
        password_version -> Integer,
        owner -> Bool,
        photo_filename -> Nullable<Text>,
        roles -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
        account_expires_at -> Nullable<Timestamp>,
        credentials_expire_at -> Nullable<Timestamp>,
        account_status -> Text,
        enabled -> Bool,
        locked -> Bool,
        locked_at -> Nullable<Timestamp>,
        locked_reason -> Nullable<Text>,
        account_id -> Nullable<Integer>,
    }
}

diesel::joinable!(addresses -> accounts (account_id));
diesel::joinable!(addresses -> brands (brand_id));
diesel::joinable!(addresses -> customers (customer_id));
diesel::joinable!(addresses -> orders (order_id));
diesel::joinable!(audit_logs -> users (user_id));
diesel::joinable!(brands -> accounts (account_id));
diesel::joinable!(categories -> accounts (account_id));
diesel::joinable!(clockwork_queries -> clockwork_requests (request_id));
diesel::joinable!(contacts -> accounts (account_id));
diesel::joinable!(contacts -> organizations (organization_id));
diesel::joinable!(customers -> accounts (account_id));
diesel::joinable!(media -> users (uploaded_by));
diesel::joinable!(order_items -> orders (order_id));
diesel::joinable!(order_items -> products (product_id));
diesel::joinable!(orders -> accounts (account_id));
diesel::joinable!(orders -> customers (customer_id));
diesel::joinable!(organizations -> accounts (account_id));
diesel::joinable!(product_categories -> categories (category_id));
diesel::joinable!(product_categories -> products (product_id));
diesel::joinable!(products -> accounts (account_id));
diesel::joinable!(products -> brands (brand_id));
diesel::joinable!(users -> accounts (account_id));

diesel::allow_tables_to_appear_in_same_query!(
    accounts,
    addresses,
    audit_logs,
    brands,
    categories,
    clockwork_queries,
    clockwork_requests,
    contacts,
    customers,
    image_jobs,
    media,
    order_items,
    orders,
    organizations,
    product_categories,
    products,
    users,
);
