use std::fs;
use std::str::FromStr;
use std::os::unix::io::IntoRawFd;
use failure::{Error, err_msg};
use nix::unistd::{self, fork, ForkResult};
use xcb::base as xbase;
use xcb::base::Connection;
use xcb::xproto;
use libc;

use crate::atoms;

pub fn into_daemon() -> Result<ForkResult, Error> {
    match fork()? {
        parent@ForkResult::Parent { .. } => Ok(parent),
        child@ForkResult::Child => {
            unistd::setsid()?;
            std::env::set_current_dir("/")?;
            // Not sure if this is safe...
            let dev_null = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open("/dev/null")?
                .into_raw_fd();
            for fd in &[libc::STDIN_FILENO, libc::STDOUT_FILENO, libc::STDERR_FILENO] {
                unistd::close(*fd)?;
                unistd::dup2(dev_null, *fd)?;
            }
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
        Ok(match *self {
            Selection::Primary => atoms::get(conn, "PRIMARY")?,
            Selection::Secondary => atoms::get(conn, "SECONDARY")?
        })
    }
}

// The selection daemon presented here is not a perfect implementation of the
// ICCCM recommendation. Currently, it does not support large transfers and does
// not verify that the requestor has received the data by monitoring for atom
// deletion. Additionally, we do not support MULTIPLE and TIMESTAMP targets even
// though those are required by the spec. However, this implements just enough
// of the spec to work well enough in practice as color codes do not tend to be
// that large. However, this assumption could of course fail with custom
// templates.

pub fn set_selection(conn: &Connection,
                     root: xproto::Window,
                     selection: &Selection,
                     string: &str) -> Result<(), Error> {
    let selection = selection.to_atom(conn)?;
    let utf8_string = atoms::get(conn, "UTF8_STRING")?;
    let targets = atoms::get(conn, "TARGETS")?;

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

    // It would be better to use a real timestamp
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

                    // We should check the event timestamp
                    
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


