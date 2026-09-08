use cxx_qt_lib::QString;

pub use shrimply_i18n::init_system_locale;

pub fn text(key: &str) -> QString {
    QString::from(shrimply_i18n::text(key).as_ref())
}

pub fn text_args(key: &str, args: &[(&str, String)]) -> QString {
    QString::from(shrimply_i18n::text_args(key, args))
}
