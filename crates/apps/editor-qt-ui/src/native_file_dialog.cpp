#include "native_file_dialog.h"

#include <QApplication>
#include <QFileDialog>
#include <QFileInfo>
#include <QWindow>

#include "cxx-qt-lib/qcoreapplication.h"

namespace shrimply {

std::unique_ptr<QGuiApplication> new_widget_application()
{
  QVector<QByteArray> arguments{ QByteArrayLiteral("shrimply-editor-qt") };
  auto *argument_data = new rust::cxxqtlib1::ApplicationArgsData(arguments);
  auto application = std::make_unique<QApplication>(argument_data->size(),
                                                     argument_data->data());
  argument_data->setParent(application.get());
  return application;
}

QUrl save_project_file_dialog(const QUrl &suggested_url,
                              const QString &title,
                              const QString &filter)
{
  QFileDialog dialog;
  const QFileInfo suggested_file(suggested_url.toLocalFile());
  dialog.setWindowTitle(title);
  dialog.setDirectory(suggested_file.absolutePath());
  dialog.selectFile(suggested_file.fileName());
  dialog.setNameFilter(filter);
  dialog.setAcceptMode(QFileDialog::AcceptSave);
  dialog.setFileMode(QFileDialog::AnyFile);
  dialog.setDefaultSuffix(QStringLiteral("shrimp"));
  dialog.setOption(QFileDialog::DontUseNativeDialog);
  dialog.setWindowModality(Qt::WindowModal);

  dialog.winId();
  if (auto *window = dialog.windowHandle()) {
    auto *parent = QGuiApplication::focusWindow();
    while (parent && parent->transientParent()) {
      parent = parent->transientParent();
    }
    window->setTransientParent(parent);
  }

  if (dialog.exec() != QDialog::Accepted || dialog.selectedUrls().isEmpty()) {
    return {};
  }
  return dialog.selectedUrls().constFirst();
}

} // namespace shrimply
