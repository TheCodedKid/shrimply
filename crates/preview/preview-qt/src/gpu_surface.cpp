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
                                            float blue, float alpha, bool dark,
                                            bool fullscreen);
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
extern "C" bool shrimply_qt_timeline_magnet();
extern "C" void shrimply_qt_timeline_set_magnet(bool enabled);
extern "C" bool shrimply_qt_timeline_beat_grid();
extern "C" void shrimply_qt_timeline_set_beat_grid(bool enabled);
extern "C" bool shrimply_qt_timeline_cut_enabled();
extern "C" void shrimply_qt_timeline_set_cut_enabled(bool enabled);
extern "C" bool shrimply_qt_timeline_overwrite_mode();
extern "C" bool shrimply_qt_timeline_block_mode();
extern "C" bool shrimply_qt_timeline_new_track_mode();
extern "C" void shrimply_qt_timeline_select_overwrite_mode();
extern "C" void shrimply_qt_timeline_select_block_mode();
extern "C" void shrimply_qt_timeline_select_new_track_mode();
extern "C" bool shrimply_qt_timeline_begin_pointer_lock(void *display, void *surface,
                                                          void *seat);
extern "C" void shrimply_qt_timeline_end_pointer_lock(bool control, bool shift);
extern "C" void shrimply_qt_preview_pointer_move(float width, float height, float x, float y);
extern "C" std::uint8_t shrimply_qt_preview_pointer_cursor();
extern "C" void shrimply_qt_preview_pointer_leave();
extern "C" bool shrimply_qt_preview_pointer_press(float width, float height, float x, float y);
extern "C" void shrimply_qt_preview_pointer_release(float width, float height, float x, float y);
extern "C" void shrimply_qt_preview_pointer_cancel();
extern "C" bool shrimply_qt_preview_guides_visible();
extern "C" void shrimply_qt_preview_set_guides_visible(bool visible);

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
        fullscreen_ = static_cast<shrimply::PreviewSurface *>(item)->fullscreenPreview();
    }

    void render() override {
        const QSize size = framebufferObject()->size();
        const QColor background = QGuiApplication::palette().color(QPalette::Window);
        if (!shrimply_qt_render_preview(
                static_cast<std::uint32_t>(size.width()),
                static_cast<std::uint32_t>(size.height()), scale_,
                background.redF(), background.greenF(), background.blueF(),
                background.alphaF(), dark_palette(), fullscreen_)) {
            qFatal("Shrimply could not render the preview with OpenGL");
        }
        QQuickOpenGLUtils::resetOpenGLState();
        update();
    }

private:
    float scale_ = 1.0f;
    bool fullscreen_ = false;
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

void update_preview_cursor(QQuickItem *item) {
    switch (shrimply_qt_preview_pointer_cursor()) {
    case 1:
        item->setCursor(QCursor(Qt::SizeHorCursor));
        break;
    case 2:
        item->setCursor(QCursor(Qt::SizeVerCursor));
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

bool TimelineSurface::magnetEnabled() const {
    return shrimply_qt_timeline_magnet();
}

void TimelineSurface::setMagnetEnabled(bool enabled) {
    if (magnetEnabled() == enabled) {
        return;
    }
    shrimply_qt_timeline_set_magnet(enabled);
    emit magnetEnabledChanged();
    update();
}

bool TimelineSurface::beatGridEnabled() const {
    return shrimply_qt_timeline_beat_grid();
}

void TimelineSurface::setBeatGridEnabled(bool enabled) {
    if (beatGridEnabled() == enabled) {
        return;
    }
    shrimply_qt_timeline_set_beat_grid(enabled);
    emit beatGridEnabledChanged();
    update();
}

bool TimelineSurface::cutEnabled() const {
    return shrimply_qt_timeline_cut_enabled();
}

void TimelineSurface::setCutEnabled(bool enabled) {
    if (cutEnabled() == enabled) {
        return;
    }
    shrimply_qt_timeline_set_cut_enabled(enabled);
    emit cursorToolChanged();
    update();
}

bool TimelineSurface::overwriteMode() const {
    return shrimply_qt_timeline_overwrite_mode();
}

bool TimelineSurface::blockMode() const {
    return shrimply_qt_timeline_block_mode();
}

bool TimelineSurface::newTrackMode() const {
    return shrimply_qt_timeline_new_track_mode();
}

void TimelineSurface::selectOverwriteMode() {
    if (overwriteMode()) {
        return;
    }
    shrimply_qt_timeline_select_overwrite_mode();
    emit dragCollisionModeChanged();
    update();
}

void TimelineSurface::selectBlockMode() {
    if (blockMode()) {
        return;
    }
    shrimply_qt_timeline_select_block_mode();
    emit dragCollisionModeChanged();
    update();
}

void TimelineSurface::selectNewTrackMode() {
    if (newTrackMode()) {
        return;
    }
    shrimply_qt_timeline_select_new_track_mode();
    emit dragCollisionModeChanged();
    update();
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
    setAcceptedMouseButtons(Qt::LeftButton);
    setAcceptHoverEvents(true);
}

QQuickFramebufferObject::Renderer *PreviewSurface::createRenderer() const {
    return new PreviewRenderer();
}

bool PreviewSurface::guidesVisible() const {
    return shrimply_qt_preview_guides_visible();
}

void PreviewSurface::setGuidesVisible(bool visible) {
    if (guidesVisible() == visible) {
        return;
    }
    shrimply_qt_preview_set_guides_visible(visible);
    emit guidesVisibleChanged();
    update();
}

bool PreviewSurface::fullscreenPreview() const {
    return fullscreen_preview_;
}

void PreviewSurface::setFullscreenPreview(bool fullscreen) {
    if (fullscreen_preview_ == fullscreen) {
        return;
    }
    fullscreen_preview_ = fullscreen;
    emit fullscreenPreviewChanged();
    update();
}

void PreviewSurface::hoverMoveEvent(QHoverEvent *event) {
    shrimply_qt_preview_pointer_move(width(), height(), event->position().x(),
                                     event->position().y());
    update_preview_cursor(this);
    event->accept();
    update();
}

void PreviewSurface::hoverLeaveEvent(QHoverEvent *event) {
    shrimply_qt_preview_pointer_leave();
    update_preview_cursor(this);
    event->accept();
    update();
}

void PreviewSurface::mousePressEvent(QMouseEvent *event) {
    if (event->button() != Qt::LeftButton ||
        !shrimply_qt_preview_pointer_press(width(), height(), event->position().x(),
                                           event->position().y())) {
        event->ignore();
        return;
    }
    forceActiveFocus(Qt::MouseFocusReason);
    setKeepMouseGrab(true);
    update_preview_cursor(this);
    event->accept();
    update();
}

void PreviewSurface::mouseMoveEvent(QMouseEvent *event) {
    shrimply_qt_preview_pointer_move(width(), height(), event->position().x(),
                                     event->position().y());
    update_preview_cursor(this);
    event->accept();
    update();
}

void PreviewSurface::mouseReleaseEvent(QMouseEvent *event) {
    if (event->button() != Qt::LeftButton) {
        event->ignore();
        return;
    }
    shrimply_qt_preview_pointer_release(width(), height(), event->position().x(),
                                        event->position().y());
    setKeepMouseGrab(false);
    update_preview_cursor(this);
    event->accept();
    update();
}

void PreviewSurface::mouseUngrabEvent() {
    shrimply_qt_preview_pointer_cancel();
    setKeepMouseGrab(false);
    update_preview_cursor(this);
    update();
}

AudioMeterSurface::AudioMeterSurface(QQuickItem *parent) : QQuickFramebufferObject(parent) {
    setMirrorVertically(true);
}

QQuickFramebufferObject::Renderer *AudioMeterSurface::createRenderer() const {
    return new AudioMeterRenderer();
}

} // namespace shrimply
