use std;
use std::str::FromStr;
use failure::{Error, err_msg};
use nix::unistd::{fork, ForkResult};
use xcb::base as xbase;
use xcb::base::Connection;
use xcb::xproto;

pub fn into_daemon() -> Result<ForkResult, Error> {
    match fork()? {
        parent@ForkResult::Parent { .. } => Ok(parent),
        child@ForkResult::Child => {
            std::env::set_current_dir("/")?;
            // TODO: Point file handles to /dev/null
            Ok(child)
        }
    }
}

pub enum Selection {
    Primary,
    Secondary
}

impl FromStr for Selection {
    type Err = Error;

    fn from_str(string: &str) -> Result<Selection, Self::Err> {
        match string {
            "primary" => Ok(Selection::Primary),
            "secondary" => Ok(Selection::Secondary),
            _ => Err(err_msg("Invalid selection"))
        }
    }
}

impl Selection {
    fn to_atom(&self, conn: &Connection) -> Result<xproto::Atom, Error> {
        Ok(match self {
            &Selection::Primary => xproto::intern_atom(conn, false, "PRIMARY")
                .get_reply()?.atom(),
            &Selection::Secondary => xproto::intern_atom(conn, false, "SECONDARY")
                .get_reply()?.atom()
        })
    }
}

pub fn set_selection(conn: &Connection,
                     root: xproto::Window,
                     selection: Selection,
                     string: &str) -> Result<(), Error> {
    let selection = selection.to_atom(conn)?;
    let utf8_string = xproto::intern_atom(conn, false, "UTF8_STRING")
        .get_reply()?.atom();
    let targets = xproto::intern_atom(conn, false, "TARGETS")
        .get_reply()?.atom();

    let window = conn.generate_id();

    xproto::create_window(conn,
                          0, // Depth
                          window, // Window
                          root, // Parent
                          0, 0, 1, 1, // Size
                          0, // Border
                          xproto::WINDOW_CLASS_INPUT_ONLY as u16, // Class
                          xbase::COPY_FROM_PARENT, // Visual
                          &[])
        .request_check()?;

    xproto::set_selection_owner(conn, window, selection, xbase::CURRENT_TIME)
        .request_check()?;

    if xproto::get_selection_owner(conn, selection).get_reply()?.owner() != window {
        return Err(err_msg("Could not take selection ownership"));
    }

    loop {
        let event = conn.wait_for_event();
        if let Some(event) = event {
            match event.response_type() {
                xproto::SELECTION_REQUEST => {
                    let event: &xproto::SelectionRequestEvent= unsafe {
                        xbase::cast_event(&event)
                    };
                    
                    let target = event.target();
                    let property = if target == utf8_string {
                        xproto::change_property(conn,
                                                xproto::PROP_MODE_REPLACE as u8,
                                                event.requestor(),
                                                event.property(),
                                                target,
                                                8,
                                                string.as_bytes())
                            .request_check()?;
                        event.property()

                    } else if target == targets {
                        xproto::change_property(conn,
                                                xproto::PROP_MODE_REPLACE as u8,
                                                event.requestor(),
                                                event.property(),
                                                target,
                                                32,
                                                &[targets, utf8_string])
                            .request_check()?;
                        event.property()
                    } else {
                        0
                    };

                    
                    let response = xproto::SelectionNotifyEvent::new(event.time(),
                                                                     event.requestor(),
                                                                     event.selection(),
                                                                     target,
                                                                     property);
                    xproto::send_event(conn, false, event.requestor(), 0, &response);
                },
                xproto::SELECTION_CLEAR => {
                    break;
                }
                _ => {}
            }
        } else {
            break;
        }
    }
    Ok(())
}


