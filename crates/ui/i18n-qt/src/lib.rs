use cxx_qt_lib::QString;

pub use shrimply_i18n_core::init_system_locale;

pub fn text(key: &str) -> QString {
    QString::from(shrimply_i18n_core::text(key).as_ref())
}

pub fn text_args(key: &str, args: &[(&str, String)]) -> QString {
    QString::from(shrimply_i18n_core::text_args(key, args))
}
