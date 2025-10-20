// pub struct Session {
//     is_authenticated: bool,
// }
//
// impl Session {
//     pub fn new(is_authenticated: bool) -> Self {
//         Session { is_authenticated }
//     }
// }
//
// pub struct SharedMainNav {
//     session: Session,
//     nav_main_admin: &'static str,
//     nav_main_public: &'static str,
// }
//
// impl SharedMainNav {
//     pub fn new(session: Session) -> Self {
//         SharedMainNav {
//             session,
//             nav_main_admin: "nav_main_admin.html",
//             nav_main_public: "nav_main_public.html",
//         }
//     }
// }
//
