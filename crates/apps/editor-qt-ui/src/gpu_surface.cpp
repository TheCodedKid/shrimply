#include "gpu_surface.h"

#include <QColor>
#include <QCursor>
#include <QFontDatabase>
#include <QGuiApplication>
#include <QHoverEvent>
#include <QIcon>
#include <QMouseEvent>
#include <QOpenGLFramebufferObject>
#include <QOpenGLFramebufferObjectFormat>
#include <QPalette>
#include <QQuickOpenGLUtils>
#include <QQuickWindow>
#include <QSGRendererInterface>
#include <QWheelEvent>
#include <QtGui/qguiapplication_platform.h>
#include <QtQml/qqml.h>

#include <cstdint>

extern "C" bool shrimply_qt_render_timeline(std::uint32_t width, std::uint32_t height,
                                             float scale, float red, float green,
                                             float blue, float alpha, bool dark);
extern "C" bool shrimply_qt_render_preview(std::uint32_t width, std::uint32_t height,
                                            float scale, float red, float green,
                                            float blue, float alpha, bool dark);
extern "C" bool shrimply_qt_render_audio_meter(std::uint32_t width, std::uint32_t height,
                                                float scale, bool dark);
extern "C" void shrimply_qt_timeline_pointer_move(float x, float y, bool control, bool shift);
extern "C" std::uint8_t shrimply_qt_timeline_pointer_cursor();
extern "C" void shrimply_qt_timeline_pointer_leave();
extern "C" void shrimply_qt_timeline_pointer_press(std::uint8_t button, float x, float y,
                                                     bool control, bool shift);
extern "C" void shrimply_qt_timeline_pointer_release(std::uint8_t button, float x, float y,
                                                       bool control, bool shift);
extern "C" void shrimply_qt_timeline_scroll(float dx, float dy, bool control, bool shift);
extern "C" bool shrimply_qt_timeline_begin_pointer_lock(void *display, void *surface,
                                                          void *seat);
extern "C" void shrimply_qt_timeline_end_pointer_lock(bool control, bool shift);

namespace {

QOpenGLFramebufferObject *make_fbo(const QSize &size) {
    QOpenGLFramebufferObjectFormat format;
    format.setAttachment(QOpenGLFramebufferObject::NoAttachment);
    format.setSamples(0);
    return new QOpenGLFramebufferObject(size, format);
}

bool dark_palette() {
    const QColor window = QGuiApplication::palette().color(QPalette::Window);
    return window.lightnessF() < 0.5;
}

class TimelineRenderer final : public QQuickFramebufferObject::Renderer {
public:
    QOpenGLFramebufferObject *createFramebufferObject(const QSize &size) override {
        return make_fbo(size);
    }

    void synchronize(QQuickFramebufferObject *item) override {
        scale_ = item->window() ? item->window()->effectiveDevicePixelRatio() : 1.0f;
    }

    void render() override {
        const QSize size = framebufferObject()->size();
        const QColor accent = QGuiApplication::palette().color(QPalette::Highlight);
        if (!shrimply_qt_render_timeline(
                static_cast<std::uint32_t>(size.width()),
                static_cast<std::uint32_t>(size.height()), scale_,
                accent.redF(), accent.greenF(), accent.blueF(), accent.alphaF(),
                dark_palette())) {
            qFatal("Shrimply could not render the timeline with OpenGL");
        }
        QQuickOpenGLUtils::resetOpenGLState();
        update();
    }

private:
    float scale_ = 1.0f;
};

class PreviewRenderer final : public QQuickFramebufferObject::Renderer {
public:
    QOpenGLFramebufferObject *createFramebufferObject(const QSize &size) override {
        return make_fbo(size);
    }

    void synchronize(QQuickFramebufferObject *item) override {
        scale_ = item->window() ? item->window()->effectiveDevicePixelRatio() : 1.0f;
    }

    void render() override {
        const QSize size = framebufferObject()->size();
        const QColor background = QGuiApplication::palette().color(QPalette::Window);
        if (!shrimply_qt_render_preview(
                static_cast<std::uint32_t>(size.width()),
                static_cast<std::uint32_t>(size.height()), scale_,
                background.redF(), background.greenF(), background.blueF(),
                background.alphaF(), dark_palette())) {
            qFatal("Shrimply could not render the preview with OpenGL");
        }
        QQuickOpenGLUtils::resetOpenGLState();
        update();
    }

private:
    float scale_ = 1.0f;
};

class AudioMeterRenderer final : public QQuickFramebufferObject::Renderer {
public:
    QOpenGLFramebufferObject *createFramebufferObject(const QSize &size) override {
        return make_fbo(size);
    }

    void synchronize(QQuickFramebufferObject *item) override {
        scale_ = item->window() ? item->window()->effectiveDevicePixelRatio() : 1.0f;
    }

    void render() override {
        const QSize size = framebufferObject()->size();
        if (!shrimply_qt_render_audio_meter(
                static_cast<std::uint32_t>(size.width()),
                static_cast<std::uint32_t>(size.height()), scale_,
                dark_palette())) {
            qFatal("Shrimply could not render the audio meter with OpenGL");
        }
        QQuickOpenGLUtils::resetOpenGLState();
        update();
    }

private:
    float scale_ = 1.0f;
};

void modifiers(const QInputEvent *event, bool &control, bool &shift) {
    control = event->modifiers().testFlag(Qt::ControlModifier);
    shift = event->modifiers().testFlag(Qt::ShiftModifier);
}

std::uint8_t pointer_button(Qt::MouseButton button) {
    return button == Qt::MiddleButton ? 1 : 0;
}

void update_timeline_cursor(QQuickItem *item) {
    switch (shrimply_qt_timeline_pointer_cursor()) {
    case 1:
    case 2:
    case 3:
        item->setCursor(QCursor(Qt::SizeHorCursor));
        break;
    case 4:
        item->setCursor(QCursor(Qt::CrossCursor));
        break;
    default:
        item->unsetCursor();
    }
}

} // namespace

namespace shrimply {

void force_opengl() {
    qputenv("QSG_RENDER_LOOP", "basic");
    QQuickWindow::setGraphicsApi(QSGRendererInterface::OpenGL);
}

void configure_icons() {
    const QString theme = dark_palette() ? QStringLiteral("breeze-dark") : QStringLiteral("breeze");
    QIcon::setThemeName(theme);
    QIcon::setFallbackThemeName(theme);
}

QString fixed_font_family() {
    return QFontDatabase::systemFont(QFontDatabase::FixedFont).family();
}

void register_gpu_surfaces() {
    qmlRegisterType<TimelineSurface>("dev.shrimply.editor", 1, 0, "TimelineSurface");
    qmlRegisterType<PreviewSurface>("dev.shrimply.editor", 1, 0, "PreviewSurface");
    qmlRegisterType<AudioMeterSurface>("dev.shrimply.editor", 1, 0, "AudioMeterSurface");
}

TimelineSurface::TimelineSurface(QQuickItem *parent) : QQuickFramebufferObject(parent) {
    setMirrorVertically(true);
    setAcceptedMouseButtons(Qt::LeftButton | Qt::MiddleButton);
    setAcceptHoverEvents(true);
}

QQuickFramebufferObject::Renderer *TimelineSurface::createRenderer() const {
    return new TimelineRenderer();
}

void TimelineSurface::hoverMoveEvent(QHoverEvent *event) {
    if (middle_mouse_grabbed_) {
        event->accept();
        return;
    }
    bool control;
    bool shift;
    modifiers(event, control, shift);
    shrimply_qt_timeline_pointer_move(event->position().x(), event->position().y(), control, shift);
    update_timeline_cursor(this);
    update();
}

void TimelineSurface::hoverLeaveEvent(QHoverEvent *event) {
    Q_UNUSED(event);
    shrimply_qt_timeline_pointer_leave();
    unsetCursor();
    update();
}

void TimelineSurface::mousePressEvent(QMouseEvent *event) {
    bool control;
    bool shift;
    modifiers(event, control, shift);
    shrimply_qt_timeline_pointer_press(pointer_button(event->button()), event->position().x(),
                                       event->position().y(), control, shift);
    if (event->button() == Qt::MiddleButton) {
        auto *wayland = qGuiApp->nativeInterface<QNativeInterface::QWaylandApplication>();
        void *surface = window() ? reinterpret_cast<void *>(window()->winId()) : nullptr;
        if (wayland && shrimply_qt_timeline_begin_pointer_lock(
                           wayland->display(), surface, wayland->seat())) {
            middle_mouse_grabbed_ = true;
            setKeepMouseGrab(true);
            setCursor(QCursor(Qt::BlankCursor));
        }
    }
    event->accept();
    update();
}

void TimelineSurface::mouseMoveEvent(QMouseEvent *event) {
    if (middle_mouse_grabbed_) {
        event->accept();
        update();
        return;
    }
    bool control;
    bool shift;
    modifiers(event, control, shift);
    shrimply_qt_timeline_pointer_move(event->position().x(), event->position().y(), control, shift);
    update_timeline_cursor(this);
    event->accept();
    update();
}

void TimelineSurface::mouseReleaseEvent(QMouseEvent *event) {
    bool control;
    bool shift;
    modifiers(event, control, shift);
    if (event->button() == Qt::MiddleButton && middle_mouse_grabbed_) {
        shrimply_qt_timeline_end_pointer_lock(control, shift);
        middle_mouse_grabbed_ = false;
        setKeepMouseGrab(false);
        unsetCursor();
    } else {
        shrimply_qt_timeline_pointer_release(pointer_button(event->button()),
                                             event->position().x(), event->position().y(),
                                             control, shift);
    }
    event->accept();
    update();
}

void TimelineSurface::mouseUngrabEvent() {
    if (!middle_mouse_grabbed_) {
        return;
    }
    shrimply_qt_timeline_end_pointer_lock(false, false);
    middle_mouse_grabbed_ = false;
    setKeepMouseGrab(false);
    unsetCursor();
    update();
}

void TimelineSurface::wheelEvent(QWheelEvent *event) {
    bool control;
    bool shift;
    modifiers(event, control, shift);
    const QPointF delta = event->pixelDelta().isNull()
                              ? QPointF(event->angleDelta()) / 120.0
                              : QPointF(event->pixelDelta()) / 120.0;
    shrimply_qt_timeline_scroll(-delta.x(), -delta.y(), control, shift);
    event->accept();
    update();
}

PreviewSurface::PreviewSurface(QQuickItem *parent) : QQuickFramebufferObject(parent) {
    setMirrorVertically(true);
}

QQuickFramebufferObject::Renderer *PreviewSurface::createRenderer() const {
    return new PreviewRenderer();
}

AudioMeterSurface::AudioMeterSurface(QQuickItem *parent) : QQuickFramebufferObject(parent) {
    setMirrorVertically(true);
}

QQuickFramebufferObject::Renderer *AudioMeterSurface::createRenderer() const {
    return new AudioMeterRenderer();
}

} // namespace shrimply
