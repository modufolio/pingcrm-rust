use anyhow::Result;
use clap::Args;
use colored::Colorize;

use crate::app::App;
use crate::router::loader::{RouteInfo, RouteLoader};
use crate::router::loaders::{AdminRoutes, ApiRoutes, AppRoutes, StaticFileRoutes, WebRoutes};

#[derive(Debug, Args)]
pub struct RouterDebugCommand {
    #[arg(long)]
    pub method: Option<String>,

    #[arg(long)]
    pub path: Option<String>,
}

impl RouterDebugCommand {
    pub async fn execute(&self) -> Result<()> {
        let groups: Vec<(&'static str, Vec<RouteInfo>)> = vec![
            ("web", <WebRoutes as RouteLoader<App>>::get_routes(&WebRoutes::new())),
            ("admin", <AdminRoutes as RouteLoader<App>>::get_routes(&AdminRoutes::new())),
            ("api", <ApiRoutes as RouteLoader<App>>::get_routes(&ApiRoutes::new())),
            ("app", <AppRoutes as RouteLoader<App>>::get_routes(&AppRoutes::new())),
            ("static", <StaticFileRoutes as RouteLoader<App>>::get_routes(&StaticFileRoutes::new())),
        ];

        let method_filter = self.method.as_deref().map(str::to_uppercase);
        let path_filter = self.path.as_deref();

        println!(
            "{:<8} {:<10} {:<40} {}",
            "Group".bold(),
            "Method".bold(),
            "Path".bold(),
            "Name".bold()
        );
        println!("{}", "-".repeat(90));

        let mut count = 0usize;
        for (group, routes) in &groups {
            for r in routes {
                if let Some(ref m) = method_filter {
                    if !r.method.split(',').any(|x| x.trim().eq_ignore_ascii_case(m)) {
                        continue;
                    }
                }
                if let Some(p) = path_filter {
                    if !r.path.contains(p) {
                        continue;
                    }
                }
                println!(
                    "{:<8} {:<10} {:<40} {}",
                    group.cyan(),
                    r.method.yellow(),
                    r.path.green(),
                    r.name
                );
                count += 1;
            }
        }

        println!();
        println!("{} routes shown", count);
        Ok(())
    }
}
