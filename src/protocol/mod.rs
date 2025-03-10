use actix_web::web;

use crate::RustusConf;

mod core;
mod creation;
pub mod extensions;
mod getting;
mod termination;

/// Configure TUS web application.
///
/// This function resolves all protocol extensions
/// provided by CLI into services and adds it to the application.
#[cfg_attr(coverage, no_coverage)]
pub fn setup(app_conf: RustusConf) -> impl Fn(&mut web::ServiceConfig) {
    move |web_app| {
        for extension in &app_conf.tus_extensions {
            match extension {
                extensions::Extensions::Creation => creation::add_extension(web_app),
                extensions::Extensions::Termination => {
                    termination::add_extension(web_app);
                }
                extensions::Extensions::Getting => {
                    getting::add_extension(web_app);
                }
                _ => {}
            }
        }
        core::add_extension(web_app);
    }
}
