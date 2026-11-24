// Include libcamera before Qt defines its emit/signals macros.
#include <libcamera/libcamera.h>
#include <libcamera/control_ids.h>
#include <libcamera/property_ids.h>
#include <libcamera/formats.h>
#include "camera.h"
#include "denoise.h"
#include <QColorSpace>
#include <QDateTime>
#include <QDir>
#include <QElapsedTimer>
#include <QFileInfo>
#include <QJsonDocument>
#include <QMutexLocker>
#include <QPainter>
#include <QSaveFile>
#include <QStandardPaths>
#include <QTransform>
#include <QUuid>
#include <QDebug>
#include <algorithm>
#include <chrono>
#include <condition_variable>
#include <deque>
#include <future>
#include <map>
#include <memory>
#include <mutex>
#include <stdexcept>
#include <vector>
#include <linux/dma-buf.h>
#include <sys/ioctl.h>
#include <sys/mman.h>

QImage PreviewProvider::requestImage(const QString &, QSize *size, const QSize &) {
    QMutexLocker lock(&frames.mutex);
    frames.requestedRevision=frames.revision;
    if (size) *size = frames.image.size();
    return frames.image;
}
QImage processedImage(const unsigned char *data, qsizetype length, int width,
                      int height, int stride, int rotation) {
    if (!data || width<=0 || height<=0 || stride<qint64(width)*3 ||
        length<qint64(stride)*(height-1)+qint64(width)*3) return {};
    QImage view(data,width,height,stride,QImage::Format_RGB888);
    QImage result=rotation ? view.transformed(QTransform().rotate(rotation)) : view.copy();
    result.setColorSpace(QColorSpace::SRgb);
    return result;
}
namespace {
void require(int result, const char *message) {
    if (result<0) throw std::runtime_error(std::string(message)+" ("+std::to_string(result)+")");
}
struct Mapping { unsigned char *base; size_t length; size_t offset; int fd; };
// libcamera emits completion on its event thread. Only the worker consumes
// requests, converts images, saves photos and requeues buffers.
class Session {
public:
    libcamera::CameraManager manager;
    std::shared_ptr<libcamera::Camera> camera;
    std::unique_ptr<libcamera::CameraConfiguration> config;
    std::unique_ptr<libcamera::FrameBufferAllocator> allocator;
    std::vector<std::unique_ptr<libcamera::Request>> requests;
    std::map<libcamera::FrameBuffer *,Mapping> mappings;
    std::mutex mutex;
    std::condition_variable condition;
    std::deque<libcamera::Request *> completed;
    bool acquired=false, started=false, managerStarted=false;
    bool exposureSupported=false;
    int rotation=270;
    // Owner-confirmed display correction on the Poco F1, applied equally to
    // the scene-graph preview and physically oriented saved pixels.
    int imageRotation() const { return (rotation+90)%360; }
    ~Session() {
        stopStream();
        if (camera) camera->requestCompleted.disconnect(this);
        if (acquired) camera->release();
        camera.reset();
        if (managerStarted) manager.stop();
    }
    void stopStream() {
        if (started) { camera->stop(); started=false; }
        { std::lock_guard<std::mutex> lock(mutex); completed.clear(); }
        requests.clear();
        for (const auto &[buffer,mapped]:mappings) { Q_UNUSED(buffer); munmap(mapped.base,mapped.length); }
        mappings.clear(); allocator.reset(); config.reset();
    }
    void complete(libcamera::Request *request) {
        if (request->status()==libcamera::Request::RequestCancelled) return;
        std::lock_guard<std::mutex> lock(mutex);
        completed.push_back(request); condition.notify_one();
    }
    void open() {
        require(manager.start(),"Cannot start libcamera"); managerStarted=true;
        for (const auto &candidate:manager.cameras()) {
            auto model=candidate->properties().get(libcamera::properties::Model);
            if (model && *model=="imx363") { camera=candidate; break; }
        }
        if (!camera) throw std::runtime_error("libcamera could not find the rear IMX363 sensor.");
        require(camera->acquire(),"Camera is busy. Close other camera apps and retry"); acquired=true;

        auto mounting=camera->properties().get(libcamera::properties::Rotation);
        if (mounting) rotation=*mounting;
        camera->requestCompleted.connect(this,&Session::complete);
        configure(false);
    }
    void configure(bool still) {
        stopStream();
        auto native=camera->properties().get(libcamera::properties::PixelArraySize);
        config=camera->generateConfiguration({still ? libcamera::StreamRole::StillCapture : libcamera::StreamRole::Viewfinder});
        if (!config || config->empty()) throw std::runtime_error("No libcamera still-image configuration.");
        auto &stream=config->at(0);
        stream.pixelFormat=libcamera::formats::BGR888;
        // 1000x750 fits the full sensor's 4x Bayer reduction with a small
        // interpolation border. Photos keep the original sensor resolution.
        stream.size=still && native ? *native : libcamera::Size(1000,750);
        stream.colorSpace=libcamera::ColorSpace::Srgb;
        stream.bufferCount=4;
        config->orientation=libcamera::orientationFromRotation(rotation);
        if (config->validate()==libcamera::CameraConfiguration::Invalid ||
            config->at(0).pixelFormat!=libcamera::formats::BGR888)
            throw std::runtime_error("libcamera cannot provide a processed RGB still image.");
        require(camera->configure(config.get()),"Cannot configure libcamera still capture");
        exposureSupported=camera->controls().find(&libcamera::controls::ExposureValue)!=camera->controls().end();
        allocator=std::make_unique<libcamera::FrameBufferAllocator>(camera);
        auto *output=config->at(0).stream();
        require(allocator->allocate(output),"Cannot allocate libcamera image buffers");
        for (const auto &buffer:allocator->buffers(output)) {
            if (buffer->planes().size()!=1) throw std::runtime_error("Unexpected processed-image plane layout.");
            const auto &plane=buffer->planes()[0];
            size_t length=size_t(plane.offset)+plane.length;
            auto *base=static_cast<unsigned char *>(mmap(nullptr,length,PROT_READ,MAP_SHARED,plane.fd.get(),0));
            if (base==MAP_FAILED) throw std::runtime_error("Cannot map libcamera processed-image buffer.");
            mappings.emplace(buffer.get(),Mapping{base,length,plane.offset,plane.fd.get()});
            auto request=camera->createRequest();
            if (!request) throw std::runtime_error("Cannot create libcamera capture request.");
            require(request->addBuffer(output,buffer.get()),"Cannot attach libcamera image buffer");
            requests.push_back(std::move(request));
        }
        qInfo() << "Camera stream:" << (still ? "photo" : "preview") << stream.size.width << stream.size.height;
        require(camera->start(),"Cannot start libcamera capture"); started=true;
        for (const auto &request:requests) require(camera->queueRequest(request.get()),"Cannot queue libcamera capture");
    }
    void controls(libcamera::Request *request, double ev) {
        if (!exposureSupported) return;
        const auto &info=camera->controls().at(&libcamera::controls::ExposureValue);
        request->controls().set(libcamera::controls::ExposureValue,
            std::clamp(float(ev),info.min().get<float>(),info.max().get<float>()));
    }
};
float gainFromMetadata(const libcamera::Request &request) {
    return request.metadata().get(libcamera::controls::AnalogueGain).value_or(4.0f);
}
QJsonObject metadata(const libcamera::Request &request) {
    QJsonObject result;
    result.insert("libcameraVersion",QString::fromStdString(libcamera::CameraManager::version()));
    result.insert("colourCalibration","Sunny IMX363 stock base CCM; measured black level; generic AWB; not full factory calibration");
    for (const auto &[id,value]:request.metadata()) {
        auto found=libcamera::controls::controls.find(id);
        QString name=found!=libcamera::controls::controls.end() ? QString::fromStdString(found->second->name()) : QString::number(id);
        result.insert(name,QString::fromStdString(value.toString()));
    }
    return result;
}
}
void CameraWorker::publish(const QImage &image) {
    {
        QMutexLocker lock(&frames.mutex);
        frames.image = image;
        ++frames.revision;
    }
    if (!frames.notificationPending.exchange(true)) emit frameAvailable();
}
void CameraWorker::save(const QImage &image, const QJsonObject &metadata) {
    QElapsedTimer encoding; encoding.start();
    QString directory = QStandardPaths::writableLocation(QStandardPaths::PicturesLocation) + "/Spectre Camera";
    if (!QDir().mkpath(directory)) { emit failed("Cannot create the Pictures folder."); return; }
    QString path = directory + "/IMG_" + QDateTime::currentDateTime().toString("yyyyMMdd_HHmmss_zzz") + "_" + QUuid::createUuid().toString(QUuid::Id128).left(8) + (lossless.load() ? ".png" : ".jpg");
    QSaveFile file(path);
    if (!file.open(QIODevice::WriteOnly) || !image.save(&file, path.endsWith(".png") ? "PNG" : "JPEG", path.endsWith(".png") ? -1 : 100) || !file.commit()) {
        emit failed("Photo could not be saved. Check available storage."); return;
    }
    QJsonObject report=metadata;
    report.insert("width",image.width()); report.insert("height",image.height());
    report.insert("processing", demo ? "demo" : "libcamera simple / software ISP");
    report.insert("file",QFileInfo(path).fileName());
    QSaveFile sidecar(path+".json");
    auto encoded=QJsonDocument(report).toJson();
    if (!sidecar.open(QIODevice::WriteOnly) || sidecar.write(encoded)!=encoded.size() || !sidecar.commit())
        qWarning() << "Photo saved, but metadata sidecar could not be written:" << path;
    qInfo() << "Photo encoding / atomic save (ms):" << encoding.elapsed();
    emit saved(path);
}

void CameraWorker::run() {
    if (demo) {
        int tick = 0;
        emit configured("Demo mode · no sensor connected",false);
        emit orientationChanged(0);
        emit ready();
        while (!isInterruptionRequested()) {
            QImage image(756,1008,QImage::Format_RGB32);
            QPainter p(&image);
            QLinearGradient sky(0,0,0,1008);
            sky.setColorAt(0,QColor("#759aa2")); sky.setColorAt(.65,QColor("#e4c6a2"));
            sky.setColorAt(1,QColor("#626f55")); p.fillRect(image.rect(),sky);
            p.setPen(Qt::NoPen); p.setBrush(QColor("#f5dfab")); p.drawEllipse(450,180,135,135);
            p.setBrush(QColor("#445c52")); p.drawPolygon(QPolygon({{0,800},{300,380},{756,1008},{0,1008}}));
            p.setBrush(QColor("#263e38")); p.drawPolygon(QPolygon({{250,1008},{620,500},{756,680},{756,1008}}));
            p.setPen(QColor("#ffffff")); p.drawText(30,960,QString("DEMO · %1").arg(++tick)); p.end();
            publish(image);
            if (captureRequested.exchange(false)) save(image);
            msleep(33);
        }
        return;
    }
    // Saving must finish before this worker object can be destroyed. The future
    // joins during unwinding too, while preview normally continues independently.
    std::future<void> saving;
    try {
        QElapsedTimer startup, silence, rate, shutter;
        startup.start(); silence.start(); rate.start();
        Session session;
        session.open();
        emit orientationChanged(session.imageRotation());
        const auto &initial=session.config->at(0);
        emit configured(QString("libcamera · %1 × %2 preview · full-resolution photos").arg(initial.size.width).arg(initial.size.height),session.exposureSupported);
        bool announced=false, still=false, resuming=false;
        unsigned warmup=0, previewFrames=0;
        std::vector<NoiseFrame> burst;
        unsigned burstAttempts=0, referenceReplacements=0;
        QJsonObject previousStill;
        const bool denoise=!qEnvironmentVariableIsSet("SPECTRE_DENOISE_OFF");
        while (!isInterruptionRequested()) {
            if (!still && captureRequested.exchange(false)) {
                shutter.restart(); session.configure(true); still=true; warmup=0; burstAttempts=0; referenceReplacements=0; burst.clear(); previousStill={}; silence.restart();
            }
            libcamera::Request *request=nullptr;
            {
                std::unique_lock<std::mutex> lock(session.mutex);
                session.condition.wait_for(lock,std::chrono::milliseconds(100),[&]{return !session.completed.empty();});
                if (!session.completed.empty()) { request=session.completed.front(); session.completed.pop_front(); }
            }
            if (!request) {
                if (silence.elapsed()>15000) throw std::runtime_error("No processed frames from libcamera. Reopen the camera to retry.");
                continue;
            }
            silence.restart();
            const auto &stream=session.config->at(0);
            auto *buffer=request->buffers().at(stream.stream());
            if (buffer->metadata().status!=libcamera::FrameMetadata::FrameSuccess)
                throw std::runtime_error("libcamera returned a damaged frame.");
            bool capture=false, stable=false;
            if (still) {
                ++warmup;
                auto current=metadata(*request);
                stable=noiseFramesCompatible(previousStill,current);
                previousStill=current;
                capture=warmup>=6 && (stable || warmup>=8 || !burst.empty());
            }
            const int captureRotation=(session.imageRotation()+rotationOffset.load())%360;
            if (!still || capture) {
                const auto &mapped=session.mappings.at(buffer);
                dma_buf_sync sync{DMA_BUF_SYNC_START|DMA_BUF_SYNC_READ};
                ioctl(mapped.fd,DMA_BUF_IOCTL_SYNC,&sync);
                QImage image=processedImage(mapped.base+mapped.offset,mapped.length-mapped.offset,
                    stream.size.width,stream.size.height,stream.stride,capture ? captureRotation : 0);
                sync.flags=DMA_BUF_SYNC_END|DMA_BUF_SYNC_READ;
                ioctl(mapped.fd,DMA_BUF_IOCTL_SYNC,&sync);
                if (image.isNull()) throw std::runtime_error("Invalid libcamera processed-image layout.");
                if (!still) {
                    if (resuming) { qInfo() << "First resumed preview (ms since shutter):" << shutter.elapsed(); resuming=false; }
                    const float gain=gainFromMetadata(*request);
                    // Own pixels before recycling. Let the software ISP start
                    // its next frame while this worker filters the preview.
                    request->reuse(libcamera::Request::ReuseBuffers);
                    session.controls(request,compensation.load());
                    require(session.camera->queueRequest(request),"Cannot recycle preview buffer");
                    publish(denoise ? denoiseChroma(image,gain,true) : image);
                    if (!announced) {
                        announced=true; qInfo() << "First preview available (ms):" << startup.elapsed(); emit ready();
                    }
                    ++previewFrames;
                    if (rate.elapsed()>=3000) {
                        qInfo() << "Preview frames/s:" << previewFrames*1000.0/rate.elapsed();
                        previewFrames=0; rate.restart();
                    }
                    continue;
                } else {
                    auto report=metadata(*request);
                    report.insert("imageRotationDegrees",captureRotation);
                    ++burstAttempts;
                    // Reconfiguration resets AWB. Never keep an early reference
                    // after a later pair settles at a different exposure/WB.
                    if (!burst.empty() && stable && !noiseFramesCompatible(burst.front().metadata,report)) {
                        burst.clear(); ++referenceReplacements;
                    }
                    if (burst.empty() || noiseFramesCompatible(burst.front().metadata,report))
                        burst.push_back({image,report});
                    // Bounded burst: at most three retained frames. Discard AE/AWB
                    // changes instead of blending mismatched exposures/colours.
                    if (denoise && burst.size()<3 && burstAttempts<8 && shutter.elapsed()<5000) {
                        request->reuse(libcamera::Request::ReuseBuffers);
                        session.controls(request,compensation.load());
                        require(session.camera->queueRequest(request),"Cannot queue burst frame");
                        continue;
                    }
                    report=burst.front().metadata;
                    report.insert("burstAttempts",int(burstAttempts));
                    report.insert("burstReferenceReplacements",int(referenceReplacements));
                    report.insert("rawNoiseFilterRequested",qEnvironmentVariable("SPECTRE_IMX363_RAW_DENOISE")=="1");
                    report.insert("measuredDefectMapRequested",qEnvironmentVariable("SPECTRE_IMX363_SUNNY_DEFECTS")=="1");
                    report.insert("lensShadingRequested",qEnvironmentVariable("SPECTRE_IMX363_SHADING"));
                    // No request or stream reference survives reconfiguration.
                    session.configure(false); still=false; resuming=true;
                    qInfo() << "Full-resolution capture / preview restart (ms):" << shutter.elapsed();
                    silence.restart(); rate.restart(); previewFrames=0;
                    if (saving.valid()) saving.get();
                    saving=std::async(std::launch::async,[this,frames=std::move(burst),report,denoise]() mutable {
                        try {
                            if (denoise) {
                                auto result=denoiseBurst(frames);
                                report.insert("noiseReduction",result.report);
                                qInfo()<<"Denoise result:"<<result.report;
                                save(result.image,report);
                            } else save(frames.front().image,report);
                        } catch(const std::exception &error) {
                            // The reference is a valid captured photo. Preserve it
                            // with explicit fallback metadata if postprocessing fails.
                            report.insert("noiseReductionError",QString::fromUtf8(error.what()));
                            save(frames.front().image,report);
                        }
                    });
                    continue;
                }
            }
            request->reuse(libcamera::Request::ReuseBuffers);
            session.controls(request,compensation.load());
            require(session.camera->queueRequest(request),"Cannot recycle libcamera capture request");
        }

    } catch (const std::exception &error) { emit failed(QString::fromUtf8(error.what())); }
}
bool cameraSelfTest() {
    // libcamera BGR888 byte order is RGB. Padding must not become image pixels.
    const unsigned char pixels[]={255,0,0, 0,255,0, 11,22, 0,0,255, 255,255,255, 33,44};
    auto image=processedImage(pixels,sizeof(pixels),2,2,8,0);
    if (image.size()!=QSize(2,2) || image.pixelColor(0,0)!=QColor(Qt::red) ||
        image.pixelColor(1,0)!=QColor(Qt::green) || image.pixelColor(0,1)!=QColor(Qt::blue)) return false;
    auto rotated=processedImage(pixels,sizeof(pixels),2,2,8,90);
    if (rotated.pixelColor(1,0)!=QColor(Qt::red)) return false;
    return processedImage(pixels,4,2,2,8,0).isNull() &&
           processedImage(nullptr,16,2,2,8,0).isNull() && image.colorSpace()==QColorSpace::SRgb;
}
