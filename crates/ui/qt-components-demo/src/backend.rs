use core::pin::Pin;
use cxx_qt_lib::{QString, QStringList};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(QStringList, modifier_values, cxx_name = "modifierValues")]
        #[qproperty(QStringList, modifier_labels, cxx_name = "modifierLabels")]
        #[qproperty(QString, expression_source, cxx_name = "expressionSource")]
        type DemoLogic = super::DemoLogicRust;

        #[qinvokable]
        #[cxx_name = "expressionOutput"]
        fn expression_output(self: &DemoLogic, source: &QString) -> QString;
    }

    impl cxx_qt::Initialize for DemoLogic {}
}

#[derive(Default)]
pub struct DemoLogicRust {
    modifier_values: QStringList,
    modifier_labels: QStringList,
    expression_source: QString,
}

impl cxx_qt::Initialize for qobject::DemoLogic {
    fn initialize(mut self: Pin<&mut Self>) {
        let names = shrimply_components_demo_core::modifier_names();
        self.as_mut().set_modifier_values(
            names
                .iter()
                .map(|name| QString::from(*name))
                .collect::<QStringList>(),
        );
        self.as_mut().set_modifier_labels(
            names
                .into_iter()
                .map(QString::from)
                .collect::<QStringList>(),
        );
        self.as_mut().set_expression_source(QString::from(
            shrimply_components_demo_core::EXPRESSION_SOURCE,
        ));
    }
}

impl qobject::DemoLogic {
    pub fn expression_output(&self, source: &QString) -> QString {
        QString::from(shrimply_components_demo_core::expression_output(
            &source.to_string(),
        ))
    }
}
