#pragma once

#include <QQuickFramebufferObject>
#include <QString>

namespace shrimply {

void force_opengl();
void configure_icons();
QString fixed_font_family();
void register_gpu_surfaces();

class TimelineSurface : public QQuickFramebufferObject {
    Q_OBJECT

public:
    explicit TimelineSurface(QQuickItem *parent = nullptr);
    Renderer *createRenderer() const override;

protected:
    void hoverMoveEvent(QHoverEvent *event) override;
    void hoverLeaveEvent(QHoverEvent *event) override;
    void mousePressEvent(QMouseEvent *event) override;
    void mouseMoveEvent(QMouseEvent *event) override;
    void mouseReleaseEvent(QMouseEvent *event) override;
    void mouseUngrabEvent() override;
    void wheelEvent(QWheelEvent *event) override;

private:
    bool middle_mouse_grabbed_ = false;
    QPointF last_pointer_position_;
};

class PreviewSurface : public QQuickFramebufferObject {
    Q_OBJECT

public:
    explicit PreviewSurface(QQuickItem *parent = nullptr);
    Renderer *createRenderer() const override;
};

class AudioMeterSurface : public QQuickFramebufferObject {
    Q_OBJECT

public:
    explicit AudioMeterSurface(QQuickItem *parent = nullptr);
    Renderer *createRenderer() const override;
};

} // namespace shrimply
