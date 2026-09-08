#pragma once

#include <QStringList>

namespace shrimply {

QStringList load_recent_colors();
void save_recent_colors(const QStringList &colors);

} // namespace shrimply
