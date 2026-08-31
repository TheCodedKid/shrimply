#include "drag_input.h"

#include <QCursor>
#include <QGuiApplication>
#include <QMouseEvent>
#include <QQuickWindow>
#include <QTextCharFormat>
#include <QTextDocument>
#include <QtGui/qguiapplication_platform.h>
#include <QtQml/qqml.h>

#include <cmath>

extern "C" bool shrimply_qt_number_begin_pointer_lock(void *display, void *surface,
                                                        void *seat);
extern "C" bool shrimply_qt_number_poll_pointer_lock(double *delta_x,
                                                       double *delta_y);
extern "C" void shrimply_qt_number_end_pointer_lock();

namespace shrimply {

class TypoSyntaxHighlighter final : public QSyntaxHighlighter {
public:
    explicit TypoSyntaxHighlighter(QTextDocument *document)
        : QSyntaxHighlighter(static_cast<QObject *>(nullptr)) {
        setDocument(document);
    }

    void setRanges(const QString &ranges) {
        ranges_.clear();
        for (const QString &range : ranges.split(',', Qt::SkipEmptyParts)) {
            const qsizetype separator = range.indexOf(':');
            if (separator < 0) {
                continue;
            }
            bool start_ok = false;
            bool length_ok = false;
            const int start = range.first(separator).toInt(&start_ok);
            const int length = range.sliced(separator + 1).toInt(&length_ok);
            if (start_ok && length_ok && start >= 0 && length > 0) {
                ranges_.append({start, length});
            }
        }
        rehighlight();
    }

protected:
    void highlightBlock(const QString &) override {
        QTextCharFormat format;
        format.setUnderlineStyle(QTextCharFormat::SpellCheckUnderline);
        format.setUnderlineColor(Qt::red);
        const int block_start = currentBlock().position();
        const int block_end = block_start + currentBlock().length();
        for (const auto &[start, length] : ranges_) {
            const int end = start + length;
            const int visible_start = std::max(start, block_start);
            const int visible_end = std::min(end, block_end);
            if (visible_start < visible_end) {
                setFormat(visible_start - block_start, visible_end - visible_start, format);
            }
        }
    }

private:
    QList<QPair<int, int>> ranges_;
};

void register_drag_input() {
    qmlRegisterType<DragInput>("dev.shrimply.components", 1, 0, "DragInput");
    qmlRegisterType<TypoHighlighter>("dev.shrimply.components", 1, 0, "TypoHighlighter");
}

DragInput::DragInput(QQuickItem *parent) : QQuickItem(parent) {
    setAcceptedMouseButtons(Qt::LeftButton);
    setCursor(QCursor(Qt::SizeHorCursor));
    poll_timer_.setInterval(8);
    connect(&poll_timer_, &QTimer::timeout, this, [this]() {
        double delta_x = 0.0;
        double delta_y = 0.0;
        if (locked_ && shrimply_qt_number_poll_pointer_lock(&delta_x, &delta_y)) {
            Q_UNUSED(delta_y);
            accumulated_x_ += delta_x;
            emit dragged(accumulated_x_);
        }
    });
}

qreal DragInput::threshold() const {
    return threshold_;
}

void DragInput::setThreshold(qreal threshold) {
    threshold_ = std::max<qreal>(0.0, threshold);
}

void DragInput::mousePressEvent(QMouseEvent *event) {
    start_x_ = event->position().x();
    accumulated_x_ = 0.0;
    pressed_ = true;
    moved_ = false;
    lock_attempted_ = false;
    emit dragStarted();
    event->accept();
}

void DragInput::mouseMoveEvent(QMouseEvent *event) {
    if (!pressed_ || locked_) {
        event->accept();
        return;
    }
    const qreal offset = event->position().x() - start_x_;
    if (!moved_ && std::abs(offset) < threshold_) {
        event->accept();
        return;
    }
    moved_ = true;
    accumulated_x_ = offset;
    emit dragged(accumulated_x_);
    if (!lock_attempted_) {
        lock_attempted_ = true;
        locked_ = beginPointerLock();
        if (locked_) {
            setKeepMouseGrab(true);
            setCursor(QCursor(Qt::BlankCursor));
            poll_timer_.start();
        }
    }
    event->accept();
}

void DragInput::mouseReleaseEvent(QMouseEvent *event) {
    finish();
    event->accept();
}

void DragInput::mouseUngrabEvent() {
    if (pressed_) {
        finish();
    }
}

void DragInput::finish() {
    pressed_ = false;
    poll_timer_.stop();
    if (locked_) {
        shrimply_qt_number_end_pointer_lock();
        locked_ = false;
        setKeepMouseGrab(false);
        setCursor(QCursor(Qt::SizeHorCursor));
    }
    if (moved_) {
        emit dragFinished();
    } else {
        emit clicked();
    }
}

bool DragInput::beginPointerLock() {
    auto *wayland = qGuiApp->nativeInterface<QNativeInterface::QWaylandApplication>();
    void *surface = window() ? reinterpret_cast<void *>(window()->winId()) : nullptr;
    return wayland && shrimply_qt_number_begin_pointer_lock(
                          wayland->display(), surface, wayland->seat());
}

TypoHighlighter::TypoHighlighter(QObject *parent) : QObject(parent) {}

TypoHighlighter::~TypoHighlighter() = default;

QQuickTextDocument *TypoHighlighter::document() const {
    return document_.data();
}

void TypoHighlighter::setDocument(QQuickTextDocument *document) {
    if (document_ == document) {
        return;
    }
    document_ = document;
    rebuild();
    emit documentChanged();
}

QString TypoHighlighter::ranges() const {
    return ranges_;
}

void TypoHighlighter::setRanges(const QString &ranges) {
    if (ranges_ == ranges) {
        return;
    }
    ranges_ = ranges;
    if (highlighter_) {
        highlighter_->setRanges(ranges_);
    }
    emit rangesChanged();
}

void TypoHighlighter::rebuild() {
    highlighter_.reset();
    if (document_ && document_->textDocument()) {
        highlighter_ = std::make_unique<TypoSyntaxHighlighter>(document_->textDocument());
        highlighter_->setRanges(ranges_);
    }
}

} // namespace shrimply
