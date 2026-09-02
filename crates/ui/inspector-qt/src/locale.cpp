#include "inspector_locale.h"

#include <QDateTime>
#include <QLocale>
#include <QTimeZone>

QString formatLocalDateTime(std::int64_t seconds) {
  const auto date = QDateTime::fromSecsSinceEpoch(
      seconds, QTimeZone::systemTimeZone());
  return date.isValid()
             ? QLocale::system().toString(date, QLocale::ShortFormat)
             : QString{};
}
