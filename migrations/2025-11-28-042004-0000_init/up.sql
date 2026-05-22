PRAGMA foreign_keys = OFF;

-- Your SQL goes here
CREATE TABLE `addresses`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`label` TEXT,
	`address_line1` TEXT NOT NULL,
	`address_line2` TEXT,
	`city` TEXT NOT NULL,
	`region` TEXT,
	`country` TEXT NOT NULL,
	`postal_code` TEXT NOT NULL,
	`phone` TEXT,
	`is_default` BOOL NOT NULL,
	`address_type` TEXT NOT NULL,
	`customer_id` INTEGER,
	`brand_id` INTEGER,
	`order_id` INTEGER,
	`account_id` INTEGER,
	`created_at` TIMESTAMP NOT NULL,
	`updated_at` TIMESTAMP NOT NULL,
	FOREIGN KEY (`customer_id`) REFERENCES `customers`(`id`),
	FOREIGN KEY (`brand_id`) REFERENCES `brands`(`id`),
	FOREIGN KEY (`order_id`) REFERENCES `orders`(`id`),
	FOREIGN KEY (`account_id`) REFERENCES `accounts`(`id`)
);

CREATE TABLE `categories`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`name` TEXT NOT NULL,
	`slug` TEXT NOT NULL,
	`description` TEXT,
	`is_visible` BOOL NOT NULL,
	`parent_id` INTEGER,
	`created_at` TIMESTAMP NOT NULL,
	`updated_at` TIMESTAMP NOT NULL,
	`deleted_at` TIMESTAMP,
	`account_id` INTEGER,
	FOREIGN KEY (`account_id`) REFERENCES `accounts`(`id`)
);

CREATE TABLE `contacts`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`first_name` TEXT NOT NULL,
	`last_name` TEXT NOT NULL,
	`email` TEXT,
	`phone` TEXT,
	`address` TEXT,
	`city` TEXT,
	`region` TEXT,
	`country` TEXT,
	`postal_code` TEXT,
	`created_at` TIMESTAMP NOT NULL,
	`updated_at` TIMESTAMP NOT NULL,
	`deleted_at` TIMESTAMP,
	`account_id` INTEGER,
	`organization_id` INTEGER,
	FOREIGN KEY (`account_id`) REFERENCES `accounts`(`id`),
	FOREIGN KEY (`organization_id`) REFERENCES `organizations`(`id`)
);

CREATE TABLE `order_items`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`order_id` INTEGER NOT NULL,
	`product_id` INTEGER NOT NULL,
	`quantity` INTEGER NOT NULL,
	`unit_price` INTEGER NOT NULL,
	`total` INTEGER NOT NULL,
	FOREIGN KEY (`order_id`) REFERENCES `orders`(`id`),
	FOREIGN KEY (`product_id`) REFERENCES `products`(`id`)
);

CREATE TABLE `organizations`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`name` TEXT NOT NULL,
	`email` TEXT,
	`phone` TEXT,
	`address` TEXT,
	`city` TEXT,
	`region` TEXT,
	`country` TEXT,
	`postal_code` TEXT,
	`created_at` TIMESTAMP NOT NULL,
	`updated_at` TIMESTAMP NOT NULL,
	`deleted_at` TIMESTAMP,
	`account_id` INTEGER,
	FOREIGN KEY (`account_id`) REFERENCES `accounts`(`id`)
);

CREATE TABLE `product_categories`(
	`product_id` INTEGER NOT NULL,
	`category_id` INTEGER NOT NULL,
	PRIMARY KEY(`product_id`, `category_id`),
	FOREIGN KEY (`product_id`) REFERENCES `products`(`id`),
	FOREIGN KEY (`category_id`) REFERENCES `categories`(`id`)
);

CREATE TABLE `users`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`email` TEXT NOT NULL,
	`first_name` TEXT NOT NULL,
	`last_name` TEXT NOT NULL,
	`password` TEXT NOT NULL,
	`password_version` INTEGER NOT NULL,
	`owner` BOOL NOT NULL,
	`photo_filename` TEXT,
	`roles` TEXT,
	`created_at` TIMESTAMP NOT NULL,
	`updated_at` TIMESTAMP NOT NULL,
	`deleted_at` TIMESTAMP,
	`account_expires_at` TIMESTAMP,
	`credentials_expire_at` TIMESTAMP,
	`account_status` TEXT NOT NULL,
	`enabled` BOOL NOT NULL,
	`locked` BOOL NOT NULL,
	`locked_at` TIMESTAMP,
	`locked_reason` TEXT,
	`account_id` INTEGER,
	FOREIGN KEY (`account_id`) REFERENCES `accounts`(`id`)
);

CREATE TABLE `media`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`filename` TEXT NOT NULL,
	`original_filename` TEXT NOT NULL,
	`file_path` TEXT NOT NULL,
	`mime_type` TEXT NOT NULL,
	`file_size` BIGINT NOT NULL,
	`width` INTEGER,
	`height` INTEGER,
	`metadata` TEXT,
	`title` TEXT,
	`alt_text` TEXT,
	`caption` TEXT,
	`is_public` BOOL NOT NULL,
	`created_at` TIMESTAMP NOT NULL,
	`updated_at` TIMESTAMP NOT NULL,
	`uploaded_by` INTEGER,
	FOREIGN KEY (`uploaded_by`) REFERENCES `users`(`id`)
);

CREATE TABLE `accounts`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`name` TEXT NOT NULL,
	`created_at` TIMESTAMP NOT NULL,
	`updated_at` TIMESTAMP NOT NULL
);

CREATE TABLE `brands`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`name` TEXT NOT NULL,
	`slug` TEXT NOT NULL,
	`description` TEXT,
	`website` TEXT,
	`is_visible` BOOL NOT NULL,
	`created_at` TIMESTAMP NOT NULL,
	`updated_at` TIMESTAMP NOT NULL,
	`deleted_at` TIMESTAMP,
	`account_id` INTEGER,
	FOREIGN KEY (`account_id`) REFERENCES `accounts`(`id`)
);

CREATE TABLE `customers`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`name` TEXT NOT NULL,
	`email` TEXT,
	`phone` TEXT,
	`account_id` INTEGER,
	`created_at` TIMESTAMP NOT NULL,
	`updated_at` TIMESTAMP NOT NULL,
	`deleted_at` TIMESTAMP,
	FOREIGN KEY (`account_id`) REFERENCES `accounts`(`id`)
);

CREATE TABLE `clockwork_requests`(
	`id` TEXT NOT NULL PRIMARY KEY,
	`version` INTEGER NOT NULL,
	`request_type` TEXT NOT NULL,
	`time` DOUBLE NOT NULL,
	`method` TEXT NOT NULL,
	`url` TEXT NOT NULL,
	`uri` TEXT NOT NULL,
	`headers` TEXT,
	`get_data` TEXT,
	`post_data` TEXT,
	`cookies` TEXT,
	`response_status` INTEGER NOT NULL,
	`response_duration` DOUBLE NOT NULL,
	`memory_usage` BIGINT NOT NULL,
	`queries_count` INTEGER NOT NULL,
	`queries_duration` DOUBLE NOT NULL,
	`slow_queries` INTEGER NOT NULL,
	`middleware` TEXT,
	`created_at` TIMESTAMP NOT NULL
);

CREATE TABLE `orders`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`number` TEXT NOT NULL,
	`status` TEXT NOT NULL,
	`currency` TEXT NOT NULL,
	`subtotal` INTEGER NOT NULL,
	`tax` INTEGER NOT NULL,
	`shipping_cost` INTEGER NOT NULL,
	`total` INTEGER NOT NULL,
	`notes` TEXT,
	`customer_id` INTEGER,
	`shipping_address_id` INTEGER,
	`billing_address_id` INTEGER,
	`account_id` INTEGER,
	`created_at` TIMESTAMP NOT NULL,
	`updated_at` TIMESTAMP NOT NULL,
	`deleted_at` TIMESTAMP,
	FOREIGN KEY (`customer_id`) REFERENCES `customers`(`id`),
	FOREIGN KEY (`account_id`) REFERENCES `accounts`(`id`)
);

CREATE TABLE `clockwork_queries`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`request_id` TEXT NOT NULL,
	`sql` TEXT NOT NULL,
	`bindings` TEXT,
	`duration` DOUBLE NOT NULL,
	`query_type` TEXT NOT NULL,
	FOREIGN KEY (`request_id`) REFERENCES `clockwork_requests`(`id`)
);

CREATE TABLE `audit_logs`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`entity` TEXT NOT NULL,
	`entity_id` TEXT NOT NULL,
	`action` TEXT NOT NULL,
	`changes` TEXT,
	`user_id` INTEGER,
	`created_at` TIMESTAMP NOT NULL,
	`updated_at` TIMESTAMP NOT NULL,
	FOREIGN KEY (`user_id`) REFERENCES `users`(`id`)
);

CREATE TABLE `products`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`name` TEXT NOT NULL,
	`slug` TEXT NOT NULL,
	`sku` TEXT,
	`barcode` TEXT,
	`description` TEXT,
	`price` INTEGER NOT NULL,
	`old_price` INTEGER,
	`cost` INTEGER,
	`quantity` INTEGER NOT NULL,
	`security_stock` INTEGER NOT NULL,
	`stock_status` TEXT NOT NULL,
	`backorder` BOOL NOT NULL,
	`requires_shipping` BOOL NOT NULL,
	`published_at` DATE,
	`is_visible` BOOL NOT NULL,
	`is_featured` BOOL NOT NULL,
	`image` TEXT,
	`brand_id` INTEGER,
	`account_id` INTEGER,
	`created_at` TIMESTAMP NOT NULL,
	`updated_at` TIMESTAMP NOT NULL,
	`deleted_at` TIMESTAMP,
	FOREIGN KEY (`brand_id`) REFERENCES `brands`(`id`),
	FOREIGN KEY (`account_id`) REFERENCES `accounts`(`id`)
);

PRAGMA foreign_keys = ON;