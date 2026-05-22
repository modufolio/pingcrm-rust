use crate::database::*;
use crate::fixtures::Fixture;
use crate::seeder::EntityFactory;
use anyhow::Result;
use async_trait::async_trait;
use diesel_async::RunQueryDsl;

pub struct EntityFixtures {
    pub load_organizations: bool,
    pub load_products: bool,
    pub load_customers: bool,
    pub organization_count: usize,
    pub product_count: usize,
    pub customer_count: usize,
}

impl EntityFixtures {
    pub fn new() -> Self {
        Self {
            load_organizations: true,
            load_products: true,
            load_customers: true,
            organization_count: 5,
            product_count: 20,
            customer_count: 10,
        }
    }

    pub fn all() -> Self {
        Self::new()
    }

    pub fn organizations_only() -> Self {
        Self {
            load_organizations: true,
            load_products: false,
            load_customers: false,
            organization_count: 10,
            product_count: 0,
            customer_count: 0,
        }
    }

    pub fn products_only() -> Self {
        Self {
            load_organizations: false,
            load_products: true,
            load_customers: false,
            organization_count: 0,
            product_count: 50,
            customer_count: 0,
        }
    }

    pub fn with_counts(mut self, orgs: usize, products: usize, customers: usize) -> Self {
        self.organization_count = orgs;
        self.product_count = products;
        self.customer_count = customers;
        self
    }

    async fn load_organizations(
        &self,
        pool: &DbPool,
        account_id: i32,
    ) -> Result<Vec<Organization>> {
        println!("Creating organizations and contacts...");

        let mut organizations = Vec::new();

        let org_names = [
            (
                "Tech Solutions Ltd",
                Some("tech@solutions.com"),
                Some("+1-555-0100"),
            ),
            (
                "Global Consulting",
                Some("info@globalconsult.com"),
                Some("+1-555-0200"),
            ),
            (
                "Innovation Partners",
                Some("hello@innovation.com"),
                Some("+1-555-0300"),
            ),
            (
                "Enterprise Systems",
                Some("contact@enterprise.com"),
                Some("+1-555-0400"),
            ),
            (
                "Digital Dynamics",
                Some("info@digitaldyn.com"),
                Some("+1-555-0500"),
            ),
            (
                "Strategic Ventures",
                Some("contact@strategic.com"),
                Some("+1-555-0600"),
            ),
            (
                "Modern Solutions",
                Some("info@modern.com"),
                Some("+1-555-0700"),
            ),
            (
                "Advanced Technologies",
                Some("hello@advanced.tech"),
                Some("+1-555-0800"),
            ),
        ];

        for (i, (name, email, phone)) in org_names.iter().take(self.organization_count).enumerate()
        {
            let mut new_org = NewOrganization::new(name.to_string());
            new_org.account_id = Some(account_id);
            new_org.email = email.map(|s| s.to_string());
            new_org.phone = phone.map(|s| s.to_string());

            let mut conn = pool.get().await?;
            let org = diesel::insert_into(schema::organizations::table)
                .values(&new_org)
                .get_result::<Organization>(&mut conn)
                .await?;

            if i < 3 {
                println!("      ✓ Organization: {}", org.name);

                let contact_data = [
                    ("John", "Smith"),
                    ("Sarah", "Johnson"),
                    ("Michael", "Davis"),
                ];

                for (j, (first, last)) in contact_data.iter().take(2).enumerate() {
                    let mut new_contact = NewContact::new(first.to_string(), last.to_string());
                    new_contact.account_id = Some(account_id);
                    new_contact.organization_id = Some(org.id);
                    new_contact.email = Some(format!(
                        "{}.{}@{}",
                        first.to_lowercase(),
                        last.to_lowercase(),
                        name.to_lowercase().replace(" ", "")
                    ));
                    new_contact.phone = Some(format!("+1-555-{:04}", 1000 + i * 10 + j));

                    let mut conn = pool.get().await?;
                    let _contact = diesel::insert_into(schema::contacts::table)
                        .values(&new_contact)
                        .get_result::<Contact>(&mut conn)
                        .await?;
                }
            }

            organizations.push(org);
        }

        println!(
            "      ✓ Created {} organizations with contacts",
            self.organization_count
        );
        Ok(organizations)
    }

    async fn load_products(&self, pool: &DbPool, account_id: i32) -> Result<()> {
        println!("Creating product catalog...");

        let brand_names = vec![
            "Premium Brand",
            "EcoLine",
            "TechPro",
            "ValueMax",
            "Elite Series",
        ];
        let mut brands = Vec::new();

        for name in brand_names {
            let mut new_brand = NewBrand::new(name.to_string());
            new_brand.account_id = Some(account_id);
            new_brand.description = Some(format!(
                "{} - Quality products for discerning customers",
                name
            ));

            let mut conn = pool.get().await?;
            let brand = diesel::insert_into(schema::brands::table)
                .values(&new_brand)
                .get_result::<Brand>(&mut conn)
                .await?;

            brands.push(brand);
        }
        println!("      ✓ Created {} brands", brands.len());

        let category_names = vec![
            "Electronics",
            "Office Supplies",
            "Furniture",
            "Software",
            "Accessories",
        ];
        let mut categories = Vec::new();

        for name in category_names {
            let mut new_category = NewCategory::new(name.to_string());
            new_category.account_id = Some(account_id);
            new_category.description = Some(format!(
                "High-quality {} for business and personal use",
                name.to_lowercase()
            ));

            let mut conn = pool.get().await?;
            let category = diesel::insert_into(schema::categories::table)
                .values(&new_category)
                .get_result::<Category>(&mut conn)
                .await?;

            categories.push(category);
        }
        println!("      ✓ Created {} categories", categories.len());

        let product_templates = vec![
            ("Wireless Mouse", "Electronics", "TechPro", 2999),
            ("Mechanical Keyboard", "Electronics", "Premium Brand", 14999),
            ("USB-C Hub", "Accessories", "TechPro", 4999),
            ("Office Chair", "Furniture", "Premium Brand", 39999),
            ("Standing Desk", "Furniture", "Elite Series", 69999),
            ("Notebook Set", "Office Supplies", "ValueMax", 1299),
            ("Pen Collection", "Office Supplies", "Premium Brand", 2499),
            ("Monitor Stand", "Accessories", "TechPro", 8999),
            ("Desk Lamp", "Office Supplies", "EcoLine", 3999),
            ("Cable Organizer", "Accessories", "ValueMax", 1599),
            ("Wireless Charger", "Electronics", "TechPro", 3499),
            ("Laptop Stand", "Accessories", "Elite Series", 7999),
            ("Ergonomic Mouse Pad", "Accessories", "EcoLine", 1999),
            ("Blue Light Glasses", "Accessories", "Premium Brand", 5999),
            ("Portable SSD", "Electronics", "TechPro", 12999),
            ("Webcam HD", "Electronics", "Premium Brand", 9999),
            ("Headphone Stand", "Accessories", "Elite Series", 2999),
            ("Desk Organizer", "Office Supplies", "ValueMax", 2299),
            ("Plant Pot", "Office Supplies", "EcoLine", 1499),
            ("Whiteboard", "Office Supplies", "Premium Brand", 8999),
        ];

        let mut created_count = 0;
        for (name, _cat_name, brand_name, price) in
            product_templates.iter().take(self.product_count)
        {
            let brand = brands.iter().find(|b| b.name == *brand_name);

            let mut new_product = NewProduct::new(name.to_string(), *price);
            new_product.account_id = Some(account_id);

            if let Some(br) = brand {
                new_product.brand_id = Some(br.id);
            }

            new_product.description =
                Some(format!("Premium {} - perfect for your office needs", name));

            let mut conn = pool.get().await?;
            let _product = diesel::insert_into(schema::products::table)
                .values(&new_product)
                .get_result::<Product>(&mut conn)
                .await?;

            created_count += 1;
        }

        println!("      ✓ Created {} products", created_count);
        Ok(())
    }

    async fn load_customers(&self, pool: &DbPool, account_id: i32) -> Result<()> {
        println!("Creating customers and addresses...");

        let customer_data = vec![
            ("Emma Thompson", "emma.thompson@email.com", "+1-555-2001"),
            (
                "Oliver Martinez",
                "oliver.martinez@email.com",
                "+1-555-2002",
            ),
            (
                "Sophia Anderson",
                "sophia.anderson@email.com",
                "+1-555-2003",
            ),
            ("Liam Garcia", "liam.garcia@email.com", "+1-555-2004"),
            ("Ava Rodriguez", "ava.rodriguez@email.com", "+1-555-2005"),
            ("Noah Wilson", "noah.wilson@email.com", "+1-555-2006"),
            (
                "Isabella Taylor",
                "isabella.taylor@email.com",
                "+1-555-2007",
            ),
            ("Mason Brown", "mason.brown@email.com", "+1-555-2008"),
            ("Mia Lee", "mia.lee@email.com", "+1-555-2009"),
            ("Ethan White", "ethan.white@email.com", "+1-555-2010"),
        ];

        let mut customers = Vec::new();

        for (i, (name, email, phone)) in customer_data.iter().take(self.customer_count).enumerate()
        {
            let mut new_customer = NewCustomer::new(name.to_string());
            new_customer.account_id = Some(account_id);
            new_customer.email = Some(email.to_string());
            new_customer.phone = Some(phone.to_string());

            let mut conn = pool.get().await?;
            let customer = diesel::insert_into(schema::customers::table)
                .values(&new_customer)
                .get_result::<Customer>(&mut conn)
                .await?;

            if i < 3 {
                println!(
                    "      ✓ Customer: {} ({})",
                    customer.name,
                    customer.email.as_deref().unwrap_or("")
                );

                let addresses = vec![
                    (
                        "billing",
                        format!("{} Main St", 100 + i * 10),
                        "Springfield",
                        "IL",
                        "62701",
                    ),
                    (
                        "shipping",
                        format!("{} Oak Ave", 200 + i * 10),
                        "Springfield",
                        "IL",
                        "62702",
                    ),
                ];

                for (addr_type, street, city, state, zip) in addresses {
                    let mut new_address = NewAddress::new(
                        addr_type.to_string(),
                        street,
                        city.to_string(),
                        "USA".to_string(),
                        zip.to_string(),
                    );
                    new_address.account_id = Some(account_id);
                    new_address.customer_id = Some(customer.id);
                    new_address.region = Some(state.to_string());

                    let mut conn = pool.get().await?;
                    let _address = diesel::insert_into(schema::addresses::table)
                        .values(&new_address)
                        .get_result::<Address>(&mut conn)
                        .await?;
                }
            }

            customers.push(customer);
        }

        println!(
            "      ✓ Created {} customers with addresses",
            self.customer_count
        );
        Ok(())
    }
}

impl Default for EntityFixtures {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Fixture for EntityFixtures {
    async fn load(&self, factory: &EntityFactory) -> Result<()> {
        let pool = factory.pool();

        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;

        let mut conn = pool.get().await?;
        let account: Account = schema::accounts::table
            .order_by(schema::accounts::created_at.asc())
            .select(Account::as_select())
            .first(&mut conn)
            .await?;

        let account_id = account.id;

        if self.load_organizations {
            self.load_organizations(pool, account_id).await?;
        }

        if self.load_products {
            self.load_products(pool, account_id).await?;
        }

        if self.load_customers {
            self.load_customers(pool, account_id).await?;
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "EntityFixtures"
    }

    fn description(&self) -> &str {
        "Create test data for business entities (organizations, products, customers)"
    }

    fn order(&self) -> i32 {
        20
    }
}
