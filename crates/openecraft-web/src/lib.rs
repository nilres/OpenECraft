//! Web pages of the OpenECraft servers: what the parts server and the project server
//! share. The scripts (`common.js`: texts, requests, the language, the sign-in and the
//! rights of the session; `access.js`: the administration of roles, grants and
//! directories), the styles (`web.css`) and the logo are served by [`routes`]; a page
//! loads them before its own scripts. Its texts are written in English and translated
//! through [`WebTexts`] (`GET /api/v1/texts?lang=de`).

mod page;

pub use page::*;
