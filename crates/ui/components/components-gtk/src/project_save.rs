use std::{
    ffi::{CStr, CString, c_char, c_ulong, c_void},
    path::PathBuf,
};

use glib::translate::from_glib;
use gtk::{gdk, gio, glib, prelude::*};
use shrimply_components_core::file_picker;
use shrimply_cross_ui_core::project_save::ProjectFormat;

const PORTAL: &str = "org.freedesktop.portal.Desktop";
const REQUEST: &str = "org.freedesktop.portal.Request";
const LABEL: &str = "Save Project As";
const DEFAULT_TIMEOUT: i32 = -1;
const GLOB_PATTERN: u32 = 0;
type Filter = (String, Vec<(u32, String)>);

unsafe extern "C" {
    fn gdk_wayland_toplevel_get_type() -> glib::ffi::GType;
    fn gdk_x11_surface_get_type() -> glib::ffi::GType;
    fn gdk_x11_surface_get_xid(surface: *mut gdk::ffi::GdkSurface) -> c_ulong;
    fn gdk_wayland_toplevel_export_handle(
        toplevel: *mut gdk::ffi::GdkToplevel,
        callback: unsafe extern "C" fn(*mut gdk::ffi::GdkToplevel, *const c_char, *mut c_void),
        data: *mut c_void,
        destroy: unsafe extern "C" fn(*mut c_void),
    ) -> glib::ffi::gboolean;
    fn gdk_wayland_toplevel_drop_exported_handle(
        toplevel: *mut gdk::ffi::GdkToplevel,
        handle: *const c_char,
    );
}

// Keep the exported surface alive until the portal is done with its handle.
struct ParentHandle {
    identifier: String,
    wayland: Option<(gdk::Surface, CString)>,
}

impl Drop for ParentHandle {
    fn drop(&mut self) {
        if let Some((surface, handle)) = &self.wayland {
            unsafe {
                gdk_wayland_toplevel_drop_exported_handle(surface.as_ptr().cast(), handle.as_ptr());
            }
        }
    }
}

async fn parent_handle(window: &gtk::Window) -> Result<ParentHandle, String> {
    let surface = window
        .surface()
        .ok_or("The parent window has no surface.")?;
    // Check the actual GDK backend, rather than the session environment.
    if surface
        .type_()
        .is_a(unsafe { from_glib(gdk_x11_surface_get_type()) })
    {
        return Ok(ParentHandle {
            identifier: format!("x11:{:x}", unsafe {
                gdk_x11_surface_get_xid(surface.as_ptr())
            }),
            wayland: None,
        });
    }
    if !surface
        .type_()
        .is_a(unsafe { from_glib(gdk_wayland_toplevel_get_type()) })
    {
        return Err("The window backend cannot parent a native file dialog.".into());
    }
    type Export = (
        gdk::Surface,
        async_channel::Sender<Result<ParentHandle, String>>,
    );
    unsafe extern "C" fn exported(
        _: *mut gdk::ffi::GdkToplevel,
        handle: *const c_char,
        data: *mut c_void,
    ) {
        let (surface, sender) = unsafe { &*data.cast::<Export>() };
        let result = if handle.is_null() {
            Err("Could not export the parent window for the native file dialog.".into())
        } else {
            let handle = unsafe { CStr::from_ptr(handle) }.to_owned();
            Ok(ParentHandle {
                identifier: format!("wayland:{}", handle.to_string_lossy()),
                wayland: Some((surface.clone(), handle)),
            })
        };
        let _ = sender.try_send(result);
    }
    unsafe extern "C" fn destroy(data: *mut c_void) {
        drop(unsafe { Box::from_raw(data.cast::<Export>()) });
    }
    let (sender, receiver) = async_channel::bounded(1);
    let closed = window.connect_unrealize({
        let sender = sender.clone();
        move |_| {
            sender.close();
        }
    });
    let requested = unsafe {
        gdk_wayland_toplevel_export_handle(
            surface.as_ptr().cast(),
            exported,
            Box::into_raw(Box::new((surface.clone(), sender))).cast(),
            destroy,
        )
    };
    let result = if requested == glib::ffi::GFALSE {
        Err("Could not export the parent window for the native file dialog.".into())
    } else {
        receiver
            .recv()
            .await
            .unwrap_or_else(|_| Err("The parent window was closed.".to_owned()))
    };
    window.disconnect(closed);
    result
}

pub async fn save_as(
    window: &gtk::Window,
    suggested: PathBuf,
) -> Result<Option<(PathBuf, ProjectFormat)>, String> {
    let connection = gio::bus_get_future(gio::BusType::Session)
        .await
        .map_err(|e| e.to_string())?;
    let parent = parent_handle(window).await;
    if !window.is_realized() {
        return Ok(None);
    }
    let parent = parent?;
    let current = ProjectFormat::from_path(&suggested);
    let filters = ProjectFormat::ALL.map(|format| {
        (
            crate::i18n::text(format.label()).into_owned(),
            vec![(GLOB_PATTERN, format!("*.{}", format.extension()))],
        )
    });
    let current_filter = &filters[ProjectFormat::ALL
        .iter()
        .position(|format| *format == current)
        .expect("known project format")];
    let token = format!("shrimply_{}", uuid::Uuid::new_v4().simple());
    let sender = connection
        .unique_name()
        .ok_or("The session bus has no unique name.")?;
    let mut path = format!(
        "/org/freedesktop/portal/desktop/request/{}/{}",
        sender.trim_start_matches(':').replace('.', "_"),
        token
    );
    let options = glib::VariantDict::new(None);
    options.insert("handle_token", token);
    options.insert("modal", true);
    options.insert("filters", filters.to_vec());
    options.insert("current_filter", current_filter);
    options.insert(
        "current_name",
        suggested
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("The suggested file name is not valid UTF-8.")?,
    );
    if let Some(folder) = file_picker::initial_folder(LABEL) {
        let folder =
            CString::new(folder.as_os_str().as_encoded_bytes()).map_err(|e| e.to_string())?;
        options.insert("current_folder", folder.as_bytes_with_nul());
    }
    let (sender, receiver) = async_channel::bounded(1);
    // Subscribe before SaveFile: a portal may respond before the method returns.
    let subscription = connection.subscribe_to_signal(
        Some(PORTAL),
        Some(REQUEST),
        Some("Response"),
        Some(&path),
        None,
        gio::DBusSignalFlags::NONE,
        {
            let sender = sender.clone();
            move |signal| {
                let _ = sender.try_send(Some(signal.parameters.clone()));
            }
        },
    );
    let closed = window.connect_unrealize(move |_| {
        let _ = sender.try_send(None);
    });
    let result = async {
        let reply = connection
            .call_future(
                Some(PORTAL),
                "/org/freedesktop/portal/desktop",
                "org.freedesktop.portal.FileChooser",
                "SaveFile",
                Some(&glib::Variant::tuple_from_iter([
                    parent.identifier.to_variant(),
                    crate::i18n::text(LABEL).as_ref().to_variant(),
                    options.end(),
                ])),
                None,
                gio::DBusCallFlags::NONE,
                DEFAULT_TIMEOUT,
            )
            .await
            .map_err(|e| e.to_string())?;
        let (returned_path,) = reply
            .get::<(glib::variant::ObjectPath,)>()
            .ok_or("The native file dialog returned an invalid request handle.")?;
        if returned_path.as_str() != path {
            path = returned_path.as_str().to_owned();
            return Err("The native file dialog returned an unexpected request handle.".into());
        }
        let Some(response) = receiver.recv().await.map_err(|e| e.to_string())? else {
            return Ok(None);
        };
        if !window.is_realized() {
            return Ok(None);
        }
        let (status, values) = response
            .get::<(u32, std::collections::HashMap<String, glib::Variant>)>()
            .ok_or("Invalid native file dialog response.")?;
        match status {
            0 => {}
            1 => return Ok(None),
            _ => return Err("The native file dialog could not complete the request.".into()),
        }
        let filter = values
            .get("current_filter")
            .and_then(|value| value.get::<Filter>())
            .ok_or("The native file dialog did not return a file type.")?;
        let format = ProjectFormat::ALL
            .into_iter()
            .zip(&filters)
            .find_map(|(format, candidate)| (*candidate == filter).then_some(format))
            .ok_or("The native file dialog returned an unknown file type.")?;
        let uris = values
            .get("uris")
            .and_then(|value| value.get::<Vec<String>>())
            .ok_or("The native file dialog did not return a file.")?;
        let [uri] = uris.as_slice() else {
            return Err("The native file dialog must return exactly one file.".into());
        };
        let path = gio::File::for_uri(uri)
            .path()
            .ok_or("The selected location does not have a local path.")?;
        file_picker::remember_file(LABEL, &path);
        Ok(Some((path, format)))
    }
    .await;
    window.disconnect(closed);
    drop(subscription);
    // Close also covers parent destruction and invalid responses; completed requests are already gone.
    connection.call(
        Some(PORTAL),
        &path,
        REQUEST,
        "Close",
        None,
        None,
        gio::DBusCallFlags::NONE,
        DEFAULT_TIMEOUT,
        None::<&gio::Cancellable>,
        |_| {},
    );
    result
}
