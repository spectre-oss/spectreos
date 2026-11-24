// Explicit hardware diagnostic. Locks only through libcamera request controls;
// never touches sensor/actuator registers or installs anything on the phone.
#include <libcamera/libcamera.h>
#include <libcamera/control_ids.h>
#include <libcamera/property_ids.h>
#include <libcamera/formats.h>
#include <QCoreApplication>
#include <QDir>
#include <QFile>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QSaveFile>
#include <chrono>
#include <array>
#include <cmath>
#include <condition_variable>
#include <deque>
#include <iostream>
#include <map>
#include <mutex>
#include <stdexcept>
#include <sys/ioctl.h>
#include <sys/mman.h>
#include <linux/dma-buf.h>

namespace {
void check(int value, const char *message) { if (value < 0) throw std::runtime_error(message); }
struct Mapping { unsigned char *data; size_t length, offset; int fd; };
struct Probe {
    libcamera::CameraManager manager;
    std::shared_ptr<libcamera::Camera> camera;
    std::unique_ptr<libcamera::CameraConfiguration> config;
    std::unique_ptr<libcamera::FrameBufferAllocator> allocator;
    std::vector<std::unique_ptr<libcamera::Request>> requests;
    std::map<libcamera::FrameBuffer *,Mapping> mappings;
    std::mutex mutex;
    std::condition_variable condition;
    std::deque<libcamera::Request *> done;
    bool acquired=false, started=false, managerStarted=false;
    ~Probe() {
        if (started) camera->stop();
        if (camera) camera->requestCompleted.disconnect(this);
        requests.clear();
        for (const auto &[buffer, m] : mappings) { (void)buffer; munmap(m.data,m.length); }
        allocator.reset(); config.reset();
        if (acquired) camera->release();
        camera.reset();
        if (managerStarted) manager.stop();
    }
    void complete(libcamera::Request *r) {
        if (r->status()==libcamera::Request::RequestCancelled) return;
        std::lock_guard<std::mutex> lock(mutex); done.push_back(r); condition.notify_one();
    }
    void open(bool full) {
        check(manager.start(),"Camera manager failed"); managerStarted=true;
        for (auto &candidate:manager.cameras())
            if (candidate->properties().get(libcamera::properties::Model).value_or("")=="imx363") camera=candidate;
        if (!camera) throw std::runtime_error("No IMX363 camera");
        check(camera->acquire(),"Camera busy; close the camera app first"); acquired=true;
        camera->requestCompleted.connect(this,&Probe::complete);
        config=camera->generateConfiguration({libcamera::StreamRole::Viewfinder});
        if (!config || config->empty()) throw std::runtime_error("No camera configuration");
        auto &s=config->at(0); s.pixelFormat=libcamera::formats::BGR888;
        s.size=full ? libcamera::Size(4032,3024) : libcamera::Size(1000,750);
        s.colorSpace=libcamera::ColorSpace::Srgb; s.bufferCount=4;
        if (config->validate()==libcamera::CameraConfiguration::Invalid || s.pixelFormat!=libcamera::formats::BGR888)
            throw std::runtime_error("No processed RGB configuration");
        check(camera->configure(config.get()),"Configure failed");
        for (const libcamera::ControlId *id : std::array<const libcamera::ControlId *,7>{&libcamera::controls::AeEnable,
             &libcamera::controls::ExposureTimeMode, &libcamera::controls::AnalogueGainMode,
             &libcamera::controls::ExposureTime, &libcamera::controls::AnalogueGain,
             &libcamera::controls::AwbEnable, &libcamera::controls::ColourGains})
            if (!camera->controls().count(id)) throw std::runtime_error("Missing manual control: "+id->name());
        const auto &exp=camera->controls().at(&libcamera::controls::ExposureTime);
        const auto &gain=camera->controls().at(&libcamera::controls::AnalogueGain);
        if (exp.min().get<int32_t>()>6000 || exp.max().get<int32_t>()<24000 ||
            gain.min().get<float>()>2 || gain.max().get<float>()<4)
            throw std::runtime_error("Diagnostic targets outside advertised sensor range");
        allocator=std::make_unique<libcamera::FrameBufferAllocator>(camera);
        check(allocator->allocate(s.stream()),"Allocate failed");
        for (const auto &b:allocator->buffers(s.stream())) {
            if (b->planes().size()!=1) throw std::runtime_error("Unexpected RGB plane count");
            const auto &p=b->planes()[0]; size_t length=size_t(p.offset)+p.length;
            auto *data=static_cast<unsigned char *>(mmap(nullptr,length,PROT_READ,MAP_SHARED,p.fd.get(),0));
            if (data==MAP_FAILED) throw std::runtime_error("mmap failed");
            mappings[b.get()]={data,length,p.offset,p.fd.get()};
            auto r=camera->createRequest(); if (!r) throw std::runtime_error("Request allocation failed");
            check(r->addBuffer(s.stream(),b.get()),"Attach failed"); requests.push_back(std::move(r));
        }
        check(camera->start(),"Start failed"); started=true;
        for (auto &r:requests) check(camera->queueRequest(r.get()),"Queue failed");
    }
    libcamera::Request *next() {
        std::unique_lock<std::mutex> lock(mutex);
        if (!condition.wait_for(lock,std::chrono::seconds(15),[&]{return !done.empty();}))
            throw std::runtime_error("Frame timeout");
        auto *r=done.front(); done.pop_front();
        for (const auto &[s,b]:r->buffers()) { (void)s;
            if (b->metadata().status!=libcamera::FrameMetadata::FrameSuccess) throw std::runtime_error("Damaged frame");
        }
        return r;
    }
    double save(libcamera::Request *r, const QString &path) {
        const auto &s=config->at(0); auto *b=r->buffers().at(s.stream()); const auto &m=mappings.at(b);
        size_t rowBytes=size_t(s.size.width)*3;
        if (s.stride<rowBytes || m.length-m.offset<size_t(s.stride)*s.size.height)
            throw std::runtime_error("Invalid RGB mapping bounds");
        QByteArray pixels; pixels.reserve(int(rowBytes*s.size.height));
        dma_buf_sync sync{DMA_BUF_SYNC_START|DMA_BUF_SYNC_READ};
        check(ioctl(m.fd,DMA_BUF_IOCTL_SYNC,&sync),"RGB CPU sync failed");
        uint64_t sum=0;
        for (unsigned y=0;y<s.size.height;++y) {
            const auto *row=m.data+m.offset+y*s.stride;
            pixels.append(reinterpret_cast<const char *>(row),int(rowBytes));
            for (size_t x=0;x<rowBytes;++x) sum+=row[x];
        }
        sync.flags=DMA_BUF_SYNC_END|DMA_BUF_SYNC_READ;
        check(ioctl(m.fd,DMA_BUF_IOCTL_SYNC,&sync),"RGB CPU sync end failed");
        QSaveFile file(path);
        QByteArray header="P6\n"+QByteArray::number(s.size.width)+" "+QByteArray::number(s.size.height)+"\n255\n";
        if (!file.open(QIODevice::WriteOnly) || file.write(header)!=header.size() ||
            file.write(pixels)!=pixels.size() || !file.commit()) throw std::runtime_error("Save failed");
        return double(sum)/(rowBytes*s.size.height);
    }
};
}
int main(int argc,char **argv) {
    QCoreApplication app(argc,argv);
    try {
        auto args=app.arguments(); int index=args.indexOf("--output");
        if (index<0 || index+1>=args.size()) throw std::runtime_error("Usage: spectre-capture-probe --output NEW_DIRECTORY [--full]");
        QString directory=args[index+1];
        if (QFile::exists(directory) || !QDir().mkpath(directory)) throw std::runtime_error("Output directory must be new");
        Probe p; p.open(args.contains("--full"));
        const int exposures[]={12000,24000,6000,12000}; const float gains[]={2,2,2,4};
        int stage=-1, seen=0, stable=0; std::array<float,2> wb{1,1}; QJsonArray results;
        auto deadline=std::chrono::steady_clock::now()+std::chrono::seconds(120);
        while (std::chrono::steady_clock::now()<deadline) {
            auto *r=p.next(); ++seen;
            const auto &md=r->metadata();
            auto exp=md.get(libcamera::controls::ExposureTime);
            auto gain=md.get(libcamera::controls::AnalogueGain);
            auto colours=md.get(libcamera::controls::ColourGains);
            if (!exp || !gain || !colours) throw std::runtime_error("Missing observed exposure/gain/WB metadata");
            if (stage==-1 && seen>=40) {
                wb={(*colours)[0],(*colours)[1]}; stage=0; seen=0;
            } else if (stage>=0 && stage<4) {
                bool match=std::abs(*exp-exposures[stage])<=150 && std::abs(*gain-gains[stage])<0.05 &&
                    std::abs((*colours)[0]-wb[0])<0.0001 && std::abs((*colours)[1]-wb[1])<0.0001;
                stable=match ? stable+1 : 0;
                if (stable>=4 && seen>=10) {
                    QJsonObject row{{"targetUs",exposures[stage]},{"observedUs",*exp},
                        {"targetGain",gains[stage]},{"observedGain",*gain},
                        {"redGain",(*colours)[0]},{"blueGain",(*colours)[1]},
                        {"framesToSettle",seen},{"sequence",int(r->sequence())}};
                    row["meanRgb"]=p.save(r,directory+QString("/manual-%1.ppm").arg(stage));
                    results.append(row); ++stage; seen=0; stable=0;
                }
                if (seen>80) throw std::runtime_error("Manual target never reached sensor metadata");
            } else if (stage==4 && seen>=40) {
                if (md.get(libcamera::controls::ExposureTimeMode)!=libcamera::controls::ExposureTimeModeAuto ||
                    md.get(libcamera::controls::AnalogueGainMode)!=libcamera::controls::AnalogueGainModeAuto ||
                    !md.get(libcamera::controls::AwbEnable).value_or(false))
                    throw std::runtime_error("Automatic mode did not resume");
                QJsonObject report{{"ok",true},{"manualFrames",results},{"autoResumedUs",*exp},
                    {"autoResumedGain",*gain},{"width",int(p.config->at(0).size.width)},
                    {"height",int(p.config->at(0).size.height)}};
                auto bytes=QJsonDocument(report).toJson(); QSaveFile file(directory+"/report.json");
                if (!file.open(QIODevice::WriteOnly) || file.write(bytes)!=bytes.size() || !file.commit())
                    throw std::runtime_error("Report save failed");
                std::cout<<bytes.constData(); return 0;
            }
            r->reuse(libcamera::Request::ReuseBuffers);
            if (stage>=0 && stage<4) {
                r->controls().set(libcamera::controls::AeEnable,false);
                r->controls().set(libcamera::controls::ExposureTime,exposures[stage]);
                r->controls().set(libcamera::controls::AnalogueGain,gains[stage]);
                r->controls().set(libcamera::controls::AwbEnable,false);
                r->controls().set(libcamera::controls::ColourGains,wb);
            } else {
                r->controls().set(libcamera::controls::AeEnable,true);
                r->controls().set(libcamera::controls::AwbEnable,true);
            }
            check(p.camera->queueRequest(r),"Requeue failed");
        }
        throw std::runtime_error("Diagnostic deadline exceeded");
    } catch (const std::exception &error) { std::cerr<<error.what()<<'\n'; return 1; }
}
