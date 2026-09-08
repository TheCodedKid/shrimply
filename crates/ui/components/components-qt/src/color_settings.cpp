#include "color_settings.h"

#include <QSettings>

namespace shrimply {

QStringList load_recent_colors() {
    QSettings settings("Shrimply", "Shrimply");
    return settings.value("colorPicker/recentColors").toStringList();
}

void save_recent_colors(const QStringList &colors) {
    QSettings settings("Shrimply", "Shrimply");
    settings.setValue("colorPicker/recentColors", colors);
}

} // namespace shrimply
