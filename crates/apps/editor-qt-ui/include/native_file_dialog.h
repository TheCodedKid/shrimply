#pragma once

#include <memory>

#include <QGuiApplication>
#include <QString>
#include <QUrl>

namespace shrimply {

std::unique_ptr<QGuiApplication> new_widget_application();
QUrl save_project_file_dialog(const QUrl &suggested_url,
                              const QString &title,
                              const QString &filter);

} // namespace shrimply
