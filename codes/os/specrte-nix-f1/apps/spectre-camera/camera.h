#pragma once
#include <QImage>
#include <QJsonObject>
#include <QMutex>
#include <QQuickImageProvider>
#include <QThread>
#include <atomic>

struct FrameStore {
    QMutex mutex;
    QImage image;
    quint64 revision = 0;
    std::atomic_bool notificationPending{false};
    std::atomic<quint64> requestedRevision{0};
};
class PreviewProvider : public QQuickImageProvider {
public:
    explicit PreviewProvider(FrameStore &frames) : QQuickImageProvider(Image), frames(frames) {}
    QImage requestImage(const QString &, QSize *size, const QSize &) override;
private:
    FrameStore &frames;
};
class CameraWorker : public QThread {
    Q_OBJECT
public:
    CameraWorker(FrameStore &frames, bool demo) : frames(frames), demo(demo) {}
    std::atomic_bool captureRequested{false};
    std::atomic<double> compensation{0};
    std::atomic_bool lossless{false};
    std::atomic<int> rotationOffset{0};
signals:
    void ready();
    void frameAvailable();
    void orientationChanged(int rotation);
    void configured(QString description, bool exposureSupported);
    void failed(QString message);
    void saved(QString path);
protected:
    void run() override;
private:
    FrameStore &frames;
    bool demo;
    void publish(const QImage &image);
    void save(const QImage &image, const QJsonObject &metadata = {});
};
// libcamera BGR888 uses R,G,B byte order on little-endian machines.
QImage processedImage(const unsigned char *data, qsizetype length, int width,
                      int height, int stride, int rotation);
bool cameraSelfTest();
