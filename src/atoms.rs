use std::sync::Mutex;
use std::collections::HashMap;
use failure::{Error, err_msg};
use xcb::Connection;
use xcb::xproto;

lazy_static! {
    static ref ATOM_CACHE: Mutex<HashMap<&'static str, xproto::Atom>> = Mutex::new(HashMap::new());
}

pub fn get(conn: &Connection, name: &'static str) -> Result<xproto::Atom, Error> {
    fn err<T>(_: T) -> Error {
        err_msg("Failed to access atom cache")
    }
    let atom = {
        ATOM_CACHE.lock()
            .map_err(err)?
            .get(name)
            .cloned()
    };
    match atom {
        Some(atom) => Ok(atom),
        None => {
            let interned = xproto::intern_atom(conn, true, name)
                .get_reply()?
                .atom();
            ATOM_CACHE.lock()
                .map_err(err)?
                .insert(name, interned);
            Ok(interned)
        }
    }
}


