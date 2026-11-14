//! The shell of the OpenECraft editor, described by tables without UI code: every
//! command ([`commands`]), its place in the ribbon and the backstage ([`ribbon`]), the
//! panes and their default arrangement ([`dock`], [`workspaces`]), the keys
//! ([`shortcuts`]), the mouse gestures of the views ([`gestures`]) and the state
//! colours ([`style`]). The editor draws the shell from them; the views of
//! `openecraft-canvas` read the keys, gestures and colours, so that they behave the
//! same wherever they are shown (the editor, the website).

pub mod commands;
pub mod dock;
pub mod gestures;
pub mod ribbon;
pub mod shortcuts;
pub mod style;
pub mod workspaces;
