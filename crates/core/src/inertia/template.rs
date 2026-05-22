use anyhow::Result;
use once_cell::sync::OnceCell;
use tera::{Context, Tera};

static TEMPLATES: OnceCell<Tera> = OnceCell::new();

pub fn init_templates() -> Result<()> {
    TEMPLATES.get_or_try_init(|| -> Result<Tera> {
        let mut tera = Tera::default();

        tera.add_raw_template("inertia.html", include_str!("../../templates/inertia.html"))?;

        Ok(tera)
    })?;

    Ok(())
}

pub fn render_template(template_name: &str, context: &Context) -> Result<String, tera::Error> {
    let tera = TEMPLATES.get().expect("Templates not initialized");
    tera.render(template_name, context)
}
