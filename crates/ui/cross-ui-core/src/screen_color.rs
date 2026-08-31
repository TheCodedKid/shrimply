use gio::prelude::DBusProxyExt;
use glib::variant::ToVariant;
use std::sync::atomic::{AtomicU32, Ordering};

const PORTAL_DESTINATION: &str = "org.freedesktop.portal.Desktop";
const PORTAL_PATH: &str = "/org/freedesktop/portal/desktop";
const SCREENSHOT_INTERFACE: &str = "org.freedesktop.portal.Screenshot";
const REQUEST_INTERFACE: &str = "org.freedesktop.portal.Request";

static PORTAL_TOKEN: AtomicU32 = AtomicU32::new(1);

pub async fn pick() -> Result<[f64; 3], String> {
    let proxy = gio::DBusProxy::for_bus_future(
        gio::BusType::Session,
        gio::DBusProxyFlags::NONE,
        None,
        PORTAL_DESTINATION,
        PORTAL_PATH,
        SCREENSHOT_INTERFACE,
    )
    .await
    .map_err(|error| error.to_string())?;
    let connection = proxy.connection();
    let sender = connection
        .unique_name()
        .ok_or_else(|| "The D-Bus connection has no unique name".to_string())?
        .trim_start_matches(':')
        .replace('.', "_");
    let token = format!(
        "shrimply_color_{}",
        PORTAL_TOKEN.fetch_add(1, Ordering::Relaxed)
    );
    let request_path = format!("/org/freedesktop/portal/desktop/request/{sender}/{token}");
    let (send, receive) = async_channel::bounded(1);
    let subscription = connection.subscribe_to_signal(
        Some(PORTAL_DESTINATION),
        Some(REQUEST_INTERFACE),
        Some("Response"),
        Some(&request_path),
        None,
        gio::DBusSignalFlags::NO_MATCH_RULE,
        move |signal| {
            let _ = send.try_send(signal.parameters.clone());
        },
    );
    let options = glib::VariantDict::new(None);
    options.insert_value("handle_token", &token.to_variant());
    proxy
        .call_future(
            "PickColor",
            Some(&glib::Variant::tuple_from_iter([
                "".to_variant(),
                options.end(),
            ])),
            gio::DBusCallFlags::NONE,
            -1,
        )
        .await
        .map_err(|error| error.to_string())?;
    let response = receive.recv().await.map_err(|error| error.to_string())?;
    drop(subscription);
    if response.child_value(0).get::<u32>() != Some(0) {
        return Err("Color selection was cancelled".to_string());
    }
    let results = response.child_value(1);
    results
        .iter()
        .find_map(|entry| {
            (entry.child_value(0).get::<String>().as_deref() == Some("color"))
                .then(|| entry.child_value(1).get::<glib::Variant>())
                .flatten()
        })
        .and_then(|value| value.get::<(f64, f64, f64)>())
        .map(|(red, green, blue)| [red, green, blue])
        .ok_or_else(|| "The portal did not return a color".to_string())
}

pub fn pick_blocking() -> Result<[f64; 3], String> {
    glib::MainContext::new().block_on(pick())
}
