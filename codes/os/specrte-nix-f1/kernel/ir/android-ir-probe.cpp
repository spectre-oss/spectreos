// SPDX-License-Identifier: GPL-2.0-only
// Bounded camera-5 capture. Reports aggregate luminance only; never exports frames.
#include <camera/NdkCameraManager.h>
#include <media/NdkImageReader.h>
#include <android/native_window.h>
#include <android/hardware_buffer.h>
#include <atomic>
#include <dlfcn.h>
#include <chrono>
#include <condition_variable>
#include <cstdio>
#include <mutex>
#include <thread>
static std::atomic<unsigned> frames{0};
static std::atomic<bool> failed{false};
static std::mutex lock;
static std::condition_variable closed_cv;
static bool closed=false;
static void disconnected(void*,ACameraDevice*) { failed=true; }
static void error(void*,ACameraDevice*,int e) { printf("camera_error=%d\n",e);failed=true; }
static void session_closed(void*,ACameraCaptureSession*) { std::lock_guard<std::mutex> g(lock);closed=true;closed_cv.notify_all(); }
static void session_state(void*,ACameraCaptureSession*) {}
static void discard(void*,AImageReader* reader) { AImage* im=nullptr; if(AImageReader_acquireLatestImage(reader,&im)==AMEDIA_OK)AImage_delete(im); }
static void image(void*,AImageReader* reader) {
 AImage* im=nullptr;
 if(AImageReader_acquireLatestImage(reader,&im)!=AMEDIA_OK)return;
 int32_t w=0,h=0,stride=0,pixel=0;uint8_t* bytes=nullptr;int len=0;
 if(AImage_getWidth(im,&w)==AMEDIA_OK && AImage_getHeight(im,&h)==AMEDIA_OK &&
    AImage_getPlaneRowStride(im,0,&stride)==AMEDIA_OK &&
    AImage_getPlanePixelStride(im,0,&pixel)==AMEDIA_OK &&
    AImage_getPlaneData(im,0,&bytes,&len)==AMEDIA_OK && w>0 && h>0 && stride>0 && pixel>0 &&
    (int64_t)(h-1)*stride+(int64_t)(w-1)*pixel<len) {
  unsigned long long sum=0;unsigned low=255,high=0;
  for(int y=0;y<h;y++)for(int x=0;x<w;x++) { unsigned v=bytes[y*stride+x*pixel];sum+=v;if(v<low)low=v;if(v>high)high=v; }
  unsigned n=++frames;
  if(n==1 || n%30==0)printf("frame=%u dimensions=%dx%d mean_y=%.2f range=%u..%u\n",n,w,h,(double)sum/(w*h),low,high);
 }
 AImage_delete(im);
}
int main() {
 setvbuf(stdout,nullptr,_IOLBF,0);
 // A standalone native client lacks the app process's Binder callback pool.
 void* binder=dlopen("libbinder_ndk.so",RTLD_NOW);
 auto start_pool=binder?reinterpret_cast<void(*)()>(dlsym(binder,"ABinderProcess_startThreadPool")):nullptr;
 if(!start_pool){puts("Binder thread pool unavailable");return 1;}
 start_pool();
 ACameraManager* manager=ACameraManager_create();
 ACameraDevice* device=nullptr;
 ACameraDevice_StateCallbacks dc{};dc.onDisconnected=disconnected;dc.onError=error;
 camera_status_t status=ACameraManager_openCamera(manager,"5",&dc,&device);
 printf("open_camera_5=%d\n",status);
 if(status!=ACAMERA_OK){ACameraManager_delete(manager);return 1;}
 AImageReader* reader=nullptr;ANativeWindow* window=nullptr;
 AImageReader* preview=nullptr;ANativeWindow* preview_window=nullptr;
 ACaptureSessionOutput* preview_output=nullptr;ACameraOutputTarget* preview_target=nullptr;
 AImageReader_ImageListener discard_listener{nullptr,discard};
 ACaptureSessionOutputContainer* container=nullptr;ACaptureSessionOutput* output=nullptr;
 ACameraOutputTarget* target=nullptr;ACaptureRequest* request=nullptr;ACameraCaptureSession* session=nullptr;
 int result=1;
 AImageReader_ImageListener listener{nullptr,image};
 ACameraCaptureSession_stateCallbacks sc{nullptr,session_closed,session_state,session_state};
 if(AImageReader_new(640,480,AIMAGE_FORMAT_YUV_420_888,3,&reader)!=AMEDIA_OK)goto done;
 if(AImageReader_setImageListener(reader,&listener)!=AMEDIA_OK || AImageReader_getWindow(reader,&window)!=AMEDIA_OK)goto done;
 if(ACaptureSessionOutputContainer_create(&container)!=ACAMERA_OK || ACaptureSessionOutput_create(window,&output)!=ACAMERA_OK)goto done;
 if(ACaptureSessionOutputContainer_add(container,output)!=ACAMERA_OK || ACameraOutputTarget_create(window,&target)!=ACAMERA_OK)goto done;
 if(ACameraDevice_createCaptureRequest(device,TEMPLATE_PREVIEW,&request)!=ACAMERA_OK || ACaptureRequest_addTarget(request,target)!=ACAMERA_OK)goto done;
 // Stock MIUI uses both a GPU preview stream and a YUV analysis stream.
 if(AImageReader_newWithUsage(640,480,AIMAGE_FORMAT_PRIVATE,AHARDWAREBUFFER_USAGE_GPU_SAMPLED_IMAGE,3,&preview)!=AMEDIA_OK)goto done;
 if(AImageReader_setImageListener(preview,&discard_listener)!=AMEDIA_OK || AImageReader_getWindow(preview,&preview_window)!=AMEDIA_OK)goto done;
 if(ACaptureSessionOutput_create(preview_window,&preview_output)!=ACAMERA_OK || ACaptureSessionOutputContainer_add(container,preview_output)!=ACAMERA_OK)goto done;
 if(ACameraOutputTarget_create(preview_window,&preview_target)!=ACAMERA_OK || ACaptureRequest_addTarget(request,preview_target)!=ACAMERA_OK)goto done;
 if(ACameraDevice_createCaptureSession(device,container,&sc,&session)!=ACAMERA_OK)goto done;
 status=ACameraCaptureSession_setRepeatingRequest(session,nullptr,1,&request,nullptr);
 printf("start_capture=%d duration_seconds=5\n",status);
 if(status==ACAMERA_OK) {
  for(int i=0;i<50&&!failed;i++)std::this_thread::sleep_for(std::chrono::milliseconds(100));
  result=frames>0&&!failed?0:1;
 }
done:
 if(session){ACameraCaptureSession_stopRepeating(session);ACameraCaptureSession_abortCaptures(session);ACameraCaptureSession_close(session);
  std::unique_lock<std::mutex> g(lock);closed_cv.wait_for(g,std::chrono::seconds(2),[]{return closed;});}
 ACameraDevice_close(device);
 if(request)ACaptureRequest_free(request);
 if(target)ACameraOutputTarget_free(target);
 if(preview_target)ACameraOutputTarget_free(preview_target);
 if(preview_output)ACaptureSessionOutput_free(preview_output);
 if(output)ACaptureSessionOutput_free(output);
 if(container)ACaptureSessionOutputContainer_free(container);
 if(reader)AImageReader_delete(reader);
 if(preview)AImageReader_delete(preview);
 ACameraManager_delete(manager);
 printf("capture_complete frames=%u success=%s\n",frames.load(),result==0?"yes":"no");return result;
}
