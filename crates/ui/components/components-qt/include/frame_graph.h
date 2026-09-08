#pragma once

#include <QQuickFramebufferObject>
#include <cstdint>

namespace shrimply {

class FrameGraphItemBase : public QQuickFramebufferObject {
    Q_OBJECT

public:
    explicit FrameGraphItemBase(QObject *parent = nullptr);
    Renderer *createRenderer() const override;
    virtual std::uintptr_t frameGraphHandle() const = 0;
};

void force_component_opengl();

} // namespace shrimply
