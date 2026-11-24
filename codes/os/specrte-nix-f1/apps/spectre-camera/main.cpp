#include "camera.h"
#include "denoise.h"
#include <QGuiApplication>
#include <QElapsedTimer>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QQuickWindow>
#include <QQuickItem>
#include <QTimer>
#include <QUrl>
#include <QFileInfo>
#include <QLockFile>
#include <QStandardPaths>
#include <QDebug>
#include <QSettings>
#include <memory>
#include <cmath>

class CameraController : public QObject {
    Q_OBJECT
    Q_PROPERTY(bool ready READ ready NOTIFY changed)
    Q_PROPERTY(bool busy READ busy NOTIFY changed)
    Q_PROPERTY(bool exposureSupported READ exposureSupported NOTIFY changed)
    Q_PROPERTY(QString message READ message NOTIFY changed)
    Q_PROPERTY(QString photo READ photo NOTIFY changed)
    Q_PROPERTY(QString preview READ preview NOTIFY previewChanged)
    Q_PROPERTY(int previewRotation READ previewRotation NOTIFY changed)
public:
    CameraController(bool demo, QObject *parent=nullptr) : QObject(parent), worker(frames,demo), demo(demo) {
        userRotation=demo ? 0 : QSettings().value("camera/rotationOffset",90).toInt();
        userRotation=((userRotation/90)%4+4)%4*90;
        worker.rotationOffset=userRotation;
        qInfo() << "Camera settings:" << QSettings().fileName() << "offset:" << userRotation;
        connect(&worker,&CameraWorker::configured,this,[this](const QString &description,bool supported){
            descriptionText=description; supportsExposure=supported; status=description; emit changed();
        });
        connect(&worker,&CameraWorker::ready,this,[this]{available=true; status=descriptionText; emit changed();});
        connect(&worker,&CameraWorker::failed,this,[this](const QString &error){
            pending=false; status=error; emit changed();
        });
        connect(&worker,&QThread::finished,this,[this]{available=false; pending=false; emit changed();});
        connect(&worker,&CameraWorker::saved,this,[this](const QString &path){
            pending=false; lastPhoto=QUrl::fromLocalFile(path).toString(); status="Saved to Pictures / Spectre Camera";
            emit changed(); emit captured();
        });
        connect(&worker,&CameraWorker::orientationChanged,this,[this](int value){rotation=value; qInfo() << "Preview angle:" << previewRotation(); emit changed();});
        connect(&worker,&CameraWorker::frameAvailable,this,[this]{
            frames.notificationPending=false;
            { QMutexLocker lock(&frames.mutex); seen=frames.revision; }
            emit previewChanged();
        });
    }
    ~CameraController() override { stop(); }
    int previewRotation() const { return (rotation+userRotation)%360; }
    bool ready() const { return available; }
    bool busy() const { return pending; }
    bool exposureSupported() const { return supportsExposure; }
    QString message() const { return status; }
    QString photo() const { return lastPhoto; }
    QString preview() const { return seen ? QString("image://viewfinder/%1").arg(seen) : QString(); }
    Q_INVOKABLE void start() {
        if (worker.isRunning()) return;
        available=false; status="Opening rear camera…"; worker.captureRequested=false;
        worker.start(); emit changed();
    }
    Q_INVOKABLE void stop() {
        worker.requestInterruption(); worker.wait(); available=false; pending=false;
        emit changed();
    }
    Q_INVOKABLE void capture() {
        if (!available || pending) return;
        pending=true; status="Saving photo…"; worker.captureRequested=true; emit changed();
    }
    Q_INVOKABLE void rotateImage() { setRotation(userRotation+90); }
    void setRotation(int degrees) {
        if (pending) return;
        userRotation=(degrees%360+360)%360;
        worker.rotationOffset=userRotation;
        if (!demo) QSettings().setValue("camera/rotationOffset",userRotation);
        qInfo() << "Image orientation (degrees):" << previewRotation();
        emit changed();
    }
    Q_INVOKABLE void setBrightness(double value) { if (supportsExposure) worker.compensation=value; }
    Q_INVOKABLE void setLossless(bool value) { worker.lossless=value; }
    FrameStore frames;
    CameraWorker worker;
signals:
    void changed();
    void previewChanged();
    void captured();
private:
    int rotation=0, userRotation=0;
    bool available=false, pending=false;
    bool demo;
    bool supportsExposure=false;
    QString descriptionText="Opening libcamera…";
    quint64 seen=0;
    QString status="Opening rear camera…",lastPhoto;
};
int main(int argc,char **argv) {
    QElapsedTimer launch; launch.start();
    // This UI has no text-entry fields. Phosh's Wayland text-input context
    // can activate the OSK when the shutter gains/loses focus during capture.
    // Compose keeps physical-key shortcuts without requesting a text session.
    // Set both selectors before Qt starts; the desktop exports QT_IM_MODULES.
    qputenv("QT_IM_MODULE", "compose");
    qputenv("QT_IM_MODULES", "compose");
    fprintf(stderr,"Spectre Camera: starting Qt\n");
    QGuiApplication app(argc,argv);
    fprintf(stderr,"Spectre Camera: Qt ready\n");
    app.setApplicationName("Spectre Camera"); app.setOrganizationName("Spectre");
    app.setDesktopFileName("org.spectre.Camera");
    const auto args=app.arguments();
    if (args.contains("--self-test")) {
        bool ok=cameraSelfTest(); qInfo() << "libcamera RGB ordering / stride / orientation / sRGB:" << (ok ? "PASS":"FAIL");
        return ok ? 0:1;
    }
    if (args.contains("--noise-self-test")) return noiseSelfTest() ? 0 : 6;
    int filterArg=args.indexOf("--denoise-file");
    if (filterArg>=0) {
        if (filterArg+2>=args.size()) return 2;
        QImage input(args[filterArg+1]);
        if (input.isNull()) return 2;
        int gainArg=args.indexOf("--noise-gain");
        bool valid=false;
        float gain=gainArg>=0 && gainArg+1<args.size() ? args[gainArg+1].toFloat(&valid) : 0;
        if (!valid || !std::isfinite(gain) || gain<1 || gain>16) { qCritical()<<"--denoise-file requires --noise-gain in [1,16]"; return 2; }
        QJsonObject meta{{"AnalogueGain",QString::number(gain)},{"ExposureTime","20000"},{"ColourGains","[ 1, 1 ]"}};
        auto result=denoiseBurst({{input,meta}});
        qInfo()<<result.report;
        return result.image.save(args[filterArg+2]) ? 0 : 3;
    }
    bool demo=args.contains("--demo");
    QLockFile lock(QStandardPaths::writableLocation(QStandardPaths::RuntimeLocation)+"/spectre-camera.lock");
    if (!lock.tryLock(0)) { qCritical() << (lock.error()==QLockFile::LockFailedError ? "Spectre Camera is already running." : "Cannot create the camera session lock. Check XDG_RUNTIME_DIR."); return 1; }
    CameraController camera(demo);
    int angleArg=args.indexOf("--rotation");
    if (angleArg>=0) {
        bool valid=false;
        int angle=angleArg+1<args.size() ? args[angleArg+1].toInt(&valid) : 0;
        if (!valid || angle%90) { qCritical() << "--rotation requires a multiple of 90"; return 2; }
        camera.setRotation(angle);
    }
    camera.start(); // Open the sensor while QML is loading.
    QQmlApplicationEngine engine;
    engine.addImageProvider("viewfinder",new PreviewProvider(camera.frames));
    engine.rootContext()->setContextProperty("camera",&camera);
    engine.rootContext()->setContextProperty("demoMode",demo);
    qInfo() << "QML screen:" << SPECTRE_QML_URL;
    engine.load(QUrl(QString::fromLatin1(SPECTRE_QML_URL)));
    if (engine.rootObjects().isEmpty()) return 1;
    auto *window=qobject_cast<QQuickWindow *>(engine.rootObjects().first());
    quint64 presentedRevision=0;
    int presentedFrames=0;
    QElapsedTimer presentationRate;
    QObject::connect(window,&QQuickWindow::frameSwapped,&app,[&]{
        auto revision=camera.frames.requestedRevision.load();
        if (!revision || revision==presentedRevision) return;
        presentedRevision=revision;
        if (!presentationRate.isValid()) {
            qInfo() << "First preview presentation (ms since launch):" << launch.elapsed();
            presentationRate.start();
        } else {
            ++presentedFrames;
            if (presentationRate.elapsed()>=3000) {
                qInfo() << "Presented preview updates/s:" << presentedFrames*1000.0/presentationRate.elapsed();
                presentedFrames=0; presentationRate.restart();
            }
        }
    },Qt::QueuedConnection);
    QObject::connect(&app,&QGuiApplication::applicationStateChanged,&camera,[&](Qt::ApplicationState state){
        if (state==Qt::ApplicationActive) camera.start(); else camera.stop();
    });
    if (args.contains("--lossless")) {
        camera.setLossless(true);
        engine.rootObjects().first()->setProperty("lossless",true);
    }
    if (args.contains("--controls")) {
        engine.rootObjects().first()->setProperty("showGrid",true);
        engine.rootObjects().first()->setProperty("showExposure",true);
    }
    if (args.contains("--capture-test") || args.contains("--hardware-test")) {
        if (!demo && !args.contains("--hardware-test")) { qCritical() << "Use --hardware-test explicitly for real captures."; return 2; }
        const bool hardware=args.contains("--hardware-test");
        if (hardware && demo) { qCritical() << "Hardware test cannot use demo mode."; return 2; }

        auto count=std::make_shared<int>(0);
        QObject::connect(&camera,&CameraController::captured,&app,[&,count,hardware]{
            QImage saved(QUrl(camera.photo()).toLocalFile());
            if (saved.isNull() || (hardware ? saved.width()<3000 || saved.height()<3000 : saved.size()!=QSize(756,1008))) { app.exit(3); return; }
            if (++*count==2) { qInfo() << "Repeated photo capture and camera reopen: PASS"; app.exit(0); }
            else {
                camera.stop(); camera.start();
                if (!hardware) QTimer::singleShot(500,&camera,&CameraController::capture);
            }
        });
        if (hardware) {
            QObject::connect(&camera,&CameraController::changed,&camera,[&]{
                if (camera.ready() && !camera.busy()) QTimer::singleShot(300,&camera,&CameraController::capture);
            });
        } else QTimer::singleShot(500,&camera,&CameraController::capture);
        QTimer::singleShot(60000,&app,[&]{ qCritical() << "Capture test timed out"; app.exit(4); });
    }
    camera.start();
    if (args.contains("--benchmark")) QTimer::singleShot(12000,&app,&QCoreApplication::quit);
    if (args.contains("--rotation-test")) QTimer::singleShot(1500,&app,[&]{
        auto *preview=window->findChild<QQuickItem *>("livePreview");
        if (!preview) { qCritical() << "Preview item missing"; app.exit(5); return; }
        auto before=window->grabWindow();
        double oldAngle=preview->rotation();
        qInfo() << "Rendered rotation before:" << oldAngle << "controller:" << camera.previewRotation();
        camera.rotateImage();
        QTimer::singleShot(400,&app,[&,preview,before,oldAngle]{
            auto after=window->grabWindow();
            qInfo() << "Rendered rotation after:" << preview->rotation() << "controller:" << camera.previewRotation();
            const bool ok=int(preview->rotation()-oldAngle+360)%360==90 &&
                int(preview->rotation())==camera.previewRotation() && !before.isNull() && before!=after;
            int output=args.indexOf("--screenshot");
            if (output>=0 && output+1<args.size()) {
                before.save(args[output+1]+"-before.png"); after.save(args[output+1]+"-after.png");
            }
            qInfo() << "Rendered quarter-turn:" << (ok ? "PASS" : "FAIL");
            app.exit(ok ? 0 : 5);
        });
    });
    if (args.contains("--smoke-test")) QTimer::singleShot(2000,&app,[&]{
        int index=args.indexOf("--screenshot");
        auto *window=qobject_cast<QQuickWindow *>(engine.rootObjects().first());
        if (index>=0 && index+1<args.size() && !window->grabWindow().save(args[index+1])) app.exit(2);
        else app.exit(0);
    });
    int result=app.exec(); camera.stop(); return result;
}
#include "main.moc"
