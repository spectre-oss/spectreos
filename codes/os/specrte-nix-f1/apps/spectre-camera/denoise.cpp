// SPDX-License-Identifier: GPL-3.0-or-later
#include "denoise.h"
#include "noise-profile.h"
#include <QColorSpace>
#include <QElapsedTimer>
#include <QJsonArray>
#include <QDebug>
#include <algorithm>
#include <array>
#include <cmath>
#include <limits>
#include <random>
#include <stdexcept>

namespace {
float number(const QJsonObject &m,const char *name,float fallback) {
    bool ok=false; float value=m.value(name).toString().toFloat(&ok);
    if (!ok && m.value(name).isDouble()) { value=float(m.value(name).toDouble()); ok=true; }
    return ok && std::isfinite(value) && value>0 ? value : fallback;
}
float gainOf(const QJsonObject &m) { return std::clamp(number(m,"AnalogueGain",4),1.0f,16.0f); }
int luminance(const uchar *p) { return (int(p[0])+2*int(p[1])+int(p[2])+2)/4; }
float sigma(float y,float gain) {
    // Approximate propagation through gamma; CCM/AWB can increase colour noise.
    // These RGB thresholds are conservative tuning, not a second calibration.
    float v=std::max(0.025f,y/255.0f);
    float linear=std::pow(v,2.2f);
    float derivative=255.0f/(2.2f*959.0f)*std::pow(std::max(linear,0.002f),1/2.2f-1);
    return std::clamp(derivative*std::sqrt(spectre::rawNoiseVariance(linear*959,gain,1)),1.0f,22.0f);
}
struct Cell { float y=0, cr=0, cb=0; };
std::vector<float> thumbnail(const QImage &im,int &w,int &h) {
    w=im.width()/4; h=im.height()/4;
    std::vector<float> out(size_t(w)*h);
    for(int y=0;y<h;++y) for(int x=0;x<w;++x) {
        int sum=0;
        for(int j=0;j<4;++j) { const uchar *p=im.constScanLine(y*4+j)+x*12;
            for(int i=0;i<4;++i) sum+=luminance(p+3*i); }
        out[size_t(y)*w+x]=sum/16.0f;
    }
    return out;
}
float match(const std::vector<float> &a,const std::vector<float> &b,int w,int h,
            int x0,int y0,int x1,int y1,int dx,int dy,int step) {
    float sum=0; int n=0;
    for(int y=y0;y<y1;y+=step) for(int x=x0;x<x1;x+=step) {
        if(x+dx<0 || x+dx>=w || y+dy<0 || y+dy>=h) continue;
        sum+=std::min(40.0f,std::abs(a[size_t(y)*w+x]-b[size_t(y+dy)*w+x+dx])); ++n;
    }
    return n>=8 ? sum/n : 1e6f;
}
struct Shift { int x=0,y=0; bool valid=false; };
std::vector<Shift> align(const QImage &ref,const QImage &other,float gain,int &cols,int &rows) {
    int w,h,w2,h2; auto a=thumbnail(ref,w,h),b=thumbnail(other,w2,h2);
    cols=(ref.width()+63)/64;rows=(ref.height()+63)/64;
    std::vector<Shift> shifts(size_t(cols)*rows);
    if(w!=w2 || h!=h2 || w<16 || h<16) return shifts;
    int gx=0,gy=0;float best=1e6f;
    struct Candidate {float cost;int x,y;};std::vector<Candidate> candidates;
    for(int dy=-12;dy<=12;++dy) for(int dx=-12;dx<=12;++dx) {
        float cost=match(a,b,w,h,12,12,w-12,h-12,dx,dy,std::clamp(std::min(w,h)/96,2,8))+0.05f*(std::abs(dx)+std::abs(dy));
        candidates.push_back({cost,dx,dy});
        if(cost<best) {best=cost;gx=dx;gy=dy;}
    }
    float second=1e6f,zero=1e6f;
    for(const auto &c:candidates) {
        if(!c.x && !c.y)zero=c.cost;
        if(std::abs(c.x-gx)>2 || std::abs(c.y-gy)>2)second=std::min(second,c.cost);
    }
    // Repeating patterns can produce a plausible but wrong motion vector.
    // Preserve identity for statistically indistinguishable static/flat scenes;
    // otherwise refuse ambiguous alignment and retain the reference pixels.
    if(second-best<1.0f) {
        if(zero<=best+0.5f) {gx=0;gy=0;}
        else return shifts;
    }
    for(int ty=0;ty<rows;++ty) for(int tx=0;tx<cols;++tx) {
        int x0=tx*16,y0=ty*16,x1=std::min(x0+16,w),y1=std::min(y0+16,h);
        float local=1e6f;int bx=gx,by=gy;
        for(int dy=gy-2;dy<=gy+2;++dy) for(int dx=gx-2;dx<=gx+2;++dx) {
            float cost=match(a,b,w,h,x0,y0,x1,y1,dx,dy,2)+0.2f*(std::abs(dx-gx)+std::abs(dy-gy));
            if(cost<local) {local=cost;bx=dx;by=dy;}
        }
        // Refine using edge samples rather than a sparse regular lattice,
        // which can miss a one-pixel boundary and choose the wrong shift.
        struct Anchor {int x,y,l;};std::vector<Anchor> anchors;
        for(int cy=ty*64;cy<std::min(ty*64+64,ref.height()-1);cy+=8)
            for(int cx=tx*64;cx<std::min(tx*64+64,ref.width()-1);cx+=8) {
                int ax=cx,ay=cy,strongest=-1;
                for(int y=std::max(1,cy);y<std::min(cy+8,ref.height()-1);++y)
                    for(int x=std::max(1,cx);x<std::min(cx+8,ref.width()-1);++x) {
                        int gradient=std::abs(luminance(ref.constScanLine(y)+3*(x+1))-luminance(ref.constScanLine(y)+3*(x-1)))+
                                     std::abs(luminance(ref.constScanLine(y+1)+3*x)-luminance(ref.constScanLine(y-1)+3*x));
                        if(gradient>strongest) {strongest=gradient;ax=x;ay=y;}
                    }
                anchors.push_back({ax,ay,luminance(ref.constScanLine(ay)+3*ax)});
                anchors.push_back({ax+1,ay,luminance(ref.constScanLine(ay)+3*(ax+1))});
                anchors.push_back({ax,ay+1,luminance(ref.constScanLine(ay+1)+3*ax)});
            }
        int px=bx*4,py=by*4;float fine=1e6f;
        for(int dy=py-3;dy<=py+3;++dy) for(int dx=px-3;dx<=px+3;++dx) {
            float sum=0;int n=0;
            for(const auto &a:anchors) {
                if(a.x+dx<0 || a.x+dx>=other.width() || a.y+dy<0 || a.y+dy>=other.height()) continue;
                sum+=std::min(40,std::abs(a.l-luminance(other.constScanLine(a.y+dy)+3*(a.x+dx))));++n;
            }
            float cost=n>=8 ? sum/n+0.05f*(std::abs(dx-px)+std::abs(dy-py)) : 1e6f;
            if(cost<fine) { fine=cost; shifts[size_t(ty)*cols+tx]={dx,dy,true}; }
        }
        float level=a[size_t(std::min(y0,h-1))*w+std::min(x0,w-1)];
        if(local>std::max(4.0f,2*sigma(level,gain)) || fine>std::max(8.0f,3*sigma(level,gain)))
            shifts[size_t(ty)*cols+tx].valid=false;
    }
    return shifts;
}
}

bool noiseFramesCompatible(const QJsonObject &a,const QJsonObject &b) {
    for(const char *key:{"AnalogueGain","ExposureTime"}) {
        float x=number(a,key,-1),y=number(b,key,-1);
        if(x<=0 || y<=0 || std::abs(x-y)/x>0.035f) return false;
    }
    // The pinned IPA serializes gains as strings. Missing WB cannot establish
    // compatibility; reject rather than blend differently coloured frames.
    auto wb=[](const QJsonObject &m) {
        QString s=m.value("ColourGains").toString();s.remove('[');s.remove(']');
        auto values=s.split(',');std::array<float,2> out{0,0};
        if(values.size()==2) for(int i=0;i<2;++i) {bool ok=false;float v=values[i].toFloat(&ok);if(ok&&std::isfinite(v)&&v>0)out[i]=v;}
        return out;
    };
    auto x=wb(a),y=wb(b);
    for(int i=0;i<2;++i) if(x[i]<=0 || y[i]<=0 || std::abs(x[i]-y[i])/x[i]>0.05f) return false;
    return true;
}

QImage denoiseChroma(const QImage &input,float gain,bool preview) {
    if(input.isNull()) return {};
    QImage image=input.convertToFormat(QImage::Format_RGB888);
    const int width=image.width(),height=image.height(),w=(width+1)/2,h=(height+1)/2;
    std::vector<Cell> grid(size_t(w)*h),filtered(grid.size());
    std::array<float,256> thresholds;
    for(int i=0;i<256;++i) thresholds[i]=std::clamp(sigma(i,gain)*(preview?1.4f:2.0f),5.0f,30.0f);
    for(int y=0;y<h;++y) for(int x=0;x<w;++x) {
        Cell c;int n=0;
        for(int j=0;j<2 && y*2+j<height;++j) for(int i=0;i<2 && x*2+i<width;++i) {
            const uchar *p=image.constScanLine(y*2+j)+(x*2+i)*3;float l=luminance(p);
            c.y+=l;c.cr+=p[0]-l;c.cb+=p[2]-l;++n;
        }
        c.y/=n;c.cr/=n;c.cb/=n;grid[size_t(y)*w+x]=c;
    }
    const int radius=preview?1:2;
    for(int y=0;y<h;++y) for(int x=0;x<w;++x) {
        const Cell &c=grid[size_t(y)*w+x];Cell sum;float weights=0;
        float t=thresholds[std::clamp(int(c.y),0,255)];
        for(int dy=-radius;dy<=radius;++dy) for(int dx=-radius;dx<=radius;++dx) {
            int xx=x+dx,yy=y+dy;if(xx<0||xx>=w||yy<0||yy>=h)continue;
            const Cell &n=grid[size_t(yy)*w+xx];
            float d=std::max(std::abs(n.cr-c.cr),std::abs(n.cb-c.cb));
            float weight=std::max(0.0f,1-std::abs(n.y-c.y)/(t*2))*std::max(0.0f,1-d/(t*3));
            weight/=1+0.2f*(dx*dx+dy*dy);
            sum.cr+=n.cr*weight;sum.cb+=n.cb*weight;weights+=weight;
        }
        filtered[size_t(y)*w+x]={c.y,sum.cr/weights,sum.cb/weights};
    }
    QImage out(width,height,QImage::Format_RGB888);out.setColorSpace(input.colorSpace());
    if(preview) {
        // The RAW preview is already averaged 4x per axis. Use one guided
        // chroma lookup per pixel here; reserve expensive interpolation for
        // saved photos. A retained source contribution limits block artifacts.
        for(int y=0;y<height;++y) {
            const uchar *src=image.constScanLine(y);uchar *dst=out.scanLine(y);
            for(int x=0;x<width;++x) {
                int l=luminance(src+3*x);float cr=src[3*x]-l,cb=src[3*x+2]-l;
                const Cell &c=filtered[size_t(y/2)*w+x/2];
                float t=thresholds[l];
                float weight=0.75f*std::max(0.0f,1-std::abs(c.y-l)/(t*2))*
                    std::max(0.0f,1-std::max(std::abs(c.cr-cr),std::abs(c.cb-cb))/(t*4));
                cr+=(c.cr-cr)*weight;cb+=(c.cb-cb)*weight;
                dst[3*x]=uchar(std::clamp(int(l+cr+0.5f),0,255));
                dst[3*x+1]=uchar(std::clamp(int(l-(cr+cb)/2+0.5f),0,255));
                dst[3*x+2]=uchar(std::clamp(int(l+cb+0.5f),0,255));
            }
        }
        return out;
    }
    for(int y=0;y<height;++y) {const uchar *src=image.constScanLine(y);uchar *dst=out.scanLine(y);
        for(int x=0;x<width;++x) {
            float l=luminance(src+3*x),cr=src[3*x]-l,cb=src[3*x+2]-l;
            float t=thresholds[std::clamp(int(l),0,255)],sr=cr*0.15f,sb=cb*0.15f,weights=0.15f;
            // Guided interpolation prevents half-resolution chroma block edges.
            float fx=(x-0.5f)/2,fy=(y-0.5f)/2;int ix=int(std::floor(fx)),iy=int(std::floor(fy));
            for(int dy=0;dy<=1;++dy) for(int dx=0;dx<=1;++dx) {
                int xx=std::clamp(ix+dx,0,w-1),yy=std::clamp(iy+dy,0,h-1);
                const Cell &c=filtered[size_t(yy)*w+xx];
                float spatial=(dx?fx-ix:1-(fx-ix))*(dy?fy-iy:1-(fy-iy));
                float d=std::max(std::abs(c.cr-cr),std::abs(c.cb-cb));
                float weight=spatial*std::max(0.0f,1-std::abs(c.y-l)/(t*2))*std::max(0.0f,1-d/(t*4));
                sr+=c.cr*weight;sb+=c.cb*weight;weights+=weight;
            }
            cr=sr/weights;cb=sb/weights;
            dst[3*x]=uchar(std::clamp(int(std::lround(l+cr)),0,255));
            dst[3*x+1]=uchar(std::clamp(int(std::lround(l-(cr+cb)/2)),0,255));
            dst[3*x+2]=uchar(std::clamp(int(std::lround(l+cb)),0,255));
        }
    }
    return out;
}

NoiseResult denoiseBurst(const std::vector<NoiseFrame> &frames) {
    if(frames.empty() || frames.size()>4) throw std::runtime_error("Invalid denoise burst size");
    QElapsedTimer clock;clock.start();
    QImage ref=frames.front().image.convertToFormat(QImage::Format_RGB888);
    if(ref.isNull() || ref.width()>4096 || ref.height()>4096) throw std::runtime_error("Invalid denoise image size");
    float gain=gainOf(frames.front().metadata);
    std::vector<QImage> accepted;std::vector<std::vector<Shift>> motion;
    int cols=0,rows=0,validTiles=0,totalTiles=0;
    for(size_t i=1;i<frames.size();++i) {
        if(frames[i].image.size()!=ref.size() || !noiseFramesCompatible(frames.front().metadata,frames[i].metadata)) continue;
        QImage candidate=frames[i].image.convertToFormat(QImage::Format_RGB888);
        auto shifts=align(ref,candidate,gain,cols,rows);int valid=0;
        for(const auto &s:shifts) valid+=s.valid;
        totalTiles+=int(shifts.size());validTiles+=valid;
        if(valid < int(shifts.size())/4) continue;
        accepted.push_back(candidate);motion.push_back(std::move(shifts));
    }
    std::array<float,256> linear,threshold;
    for(int i=0;i<256;++i) {float s=i/255.0f;linear[i]=s<=0.04045f?s/12.92f:std::pow((s+0.055f)/1.055f,2.4f);threshold[i]=std::max(12.0f,4*sigma(i,gain));}
    std::array<uchar,65536> gamma;
    for(int i=0;i<65536;++i) {float s=i/65535.0f;gamma[i]=uchar(std::clamp(int(std::lround(255*(s<=0.0031308f?12.92f*s:1.055f*std::pow(s,1/2.4f)-0.055f))),0,255));}
    QImage merged=ref.copy();quint64 used=0;
    for(int y=0;y<ref.height();++y) {const uchar *r=ref.constScanLine(y);uchar *out=merged.scanLine(y);
        for(int x=0;x<ref.width();++x) {
            const uchar *p=r+3*x;float sum[3]={linear[p[0]],linear[p[1]],linear[p[2]]},weight=1;
            int l=luminance(p);float t=threshold[l];
            for(size_t i=0;i<accepted.size();++i) {
                const Shift &s=motion[i][size_t(y/64)*cols+x/64];int xx=x+s.x,yy=y+s.y;
                if(!s.valid || xx<0||xx>=ref.width()||yy<0||yy>=ref.height())continue;
                const uchar *q=accepted[i].constScanLine(yy)+3*xx;
                // Per-pixel robust rejection protects moving objects/occlusions
                // even when their tile has an acceptable background match.
                int d=std::max({std::abs(int(q[0])-p[0]),std::abs(int(q[1])-p[1]),std::abs(int(q[2])-p[2])});
                float k=std::clamp(2-d/(t*1.5f),0.0f,1.0f)*std::clamp(2-std::abs(luminance(q)-l)/t,0.0f,1.0f);
                if(k>0.05f)++used;
                for(int c=0;c<3;++c)sum[c]+=linear[q[c]]*k;
                weight+=k;
            }
            for(int c=0;c<3;++c)out[3*x+c]=gamma[std::clamp(int(std::lround(sum[c]/weight*65535)),0,65535)];
        }
    }
    NoiseResult result;
    result.image=denoiseChroma(merged,gain,false);
    result.report={{"algorithm","motion-gated linear burst + guided chroma v1"},
        {"inputFrames",int(frames.size())},{"alignedFrames",int(accepted.size()+1)},
        {"validTiles",validTiles},{"testedTiles",totalTiles},{"blendedPixelSamples",double(used)},
        {"gain",gain},{"processingMs",double(clock.elapsed())},
        {"profile","IMX363 Sunny temporal RAW pairs, 2026-10-20; limited gain/lighting coverage"}};
    return result;
}

bool noiseSelfTest() {
    std::mt19937 random(42);std::normal_distribution<float> noise(0,9);
    QImage clean(256,192,QImage::Format_RGB888);clean.setColorSpace(QColorSpace::SRgb);
    for(int y=0;y<clean.height();++y)for(int x=0;x<clean.width();++x) {
        auto *p=clean.scanLine(y)+3*x;int v=x<128?70:170;
        p[0]=v;p[1]=v;p[2]=v;
    }
    QImage noisy=clean.copy();for(int y=0;y<noisy.height();++y)for(int x=0;x<noisy.width()*3;++x)
        noisy.scanLine(y)[x]=uchar(std::clamp(int(noisy.constScanLine(y)[x]+noise(random)),0,255));
    auto filtered=denoiseChroma(noisy,8,false);
    auto preview=denoiseChroma(noisy,8,true);
    double before=0,after=0,luma=0,previewError=0;int n=0;
    for(int y=8;y<180;++y)for(int x=8;x<120;++x) {
        auto *a=noisy.constScanLine(y)+3*x;auto *b=filtered.constScanLine(y)+3*x;
        before+=std::pow(int(a[0])-a[1],2)+std::pow(int(a[2])-a[1],2);
        after+=std::pow(int(b[0])-b[1],2)+std::pow(int(b[2])-b[1],2);
        luma+=std::abs(luminance(a)-luminance(b));
        auto *v=preview.constScanLine(y)+3*x;
        previewError+=std::pow(int(v[0])-v[1],2)+std::pow(int(v[2])-v[1],2);++n;
    }
    if(after>=before*0.5 || previewError>=before*0.65 || luma/n>1) {qCritical()<<"chroma reduction/luma"<<after/before<<luma/n;return false;}
    QJsonObject metadata{{"AnalogueGain","8"},{"ExposureTime","20000"},{"ColourGains","[ 1.5, 1.2 ]"}};
    auto bad=metadata;bad["ExposureTime"]="40000";
    if(noiseFramesCompatible(metadata,bad) || noiseFramesCompatible({},{}))return false;
    bad=metadata;bad["ColourGains"]="[ 1, 1 ]";
    if(noiseFramesCompatible(metadata,bad))return false;
    std::vector<NoiseFrame> frames;
    for(int i=0;i<4;++i) {
        QImage im=clean.copy();
        for(int y=0;y<im.height();++y)for(int x=0;x<im.width()*3;++x)
            im.scanLine(y)[x]=uchar(std::clamp(int(im.constScanLine(y)[x]+noise(random)),0,255));
        frames.push_back({im,metadata});
    }
    auto result=denoiseBurst(frames);
    double oldError=0,newError=0;
    for(int y=8;y<180;++y)for(int x=8;x<120;++x) {
        int a=luminance(frames[0].image.constScanLine(y)+3*x)-70;
        int b=luminance(result.image.constScanLine(y)+3*x)-70;
        oldError+=a*a;newError+=b*b;
    }
    if(result.report["alignedFrames"].toInt()<2 || newError>=oldError*0.65)return false;
    // Flat chromatic fields and small odd sizes must survive filtering.
    QImage odd(7,5,QImage::Format_RGB888);odd.fill(QColor(180,80,30));
    auto flat=denoiseChroma(odd,16,true);
    if(flat.size()!=odd.size() || std::abs(flat.pixelColor(3,2).red()-180)>1)return false;
    // Known camera translation with texture: alignment must recover the shift.
    QImage textured(256,192,QImage::Format_RGB888), shifted(256,192,QImage::Format_RGB888);
    for(int y=0;y<192;++y)for(int x=0;x<256;++x) {
        int value=70+((x/7+y/11)%3)*35+(x*13+y*17)%13;
        auto *p=textured.scanLine(y)+3*x;p[0]=p[1]=p[2]=uchar(value);
    }
    shifted.fill(Qt::black);
    for(int y=0;y<187;++y)for(int x=0;x<250;++x)
        std::copy_n(textured.constScanLine(y)+3*x,3,shifted.scanLine(y+5)+3*(x+6));
    int columns=0,rows=0;auto shifts=align(textured,shifted,4,columns,rows);
    int correct=0,wrong=0;
    for(const auto &s:shifts)if(s.valid) {if(s.x==6 && s.y==5)++correct;else++wrong;}
    if(wrong>0 && correct<int(shifts.size())/2) {qCritical()<<"ambiguous pattern not rejected";return false;}
    // A non-repeating, lower-frequency texture must actually align (rather
    // than pass the test by rejecting every frame).
    for(int y=0;y<192;++y)for(int x=0;x<256;++x) {
        unsigned hash=(unsigned(x/19)*73856093u)^(unsigned(y/23)*19349663u);
        auto *p=textured.scanLine(y)+3*x;p[0]=p[1]=p[2]=uchar(50+hash%150);
    }
    shifted.fill(Qt::black);
    for(int y=0;y<187;++y)for(int x=0;x<250;++x)
        std::copy_n(textured.constScanLine(y)+3*x,3,shifted.scanLine(y+5)+3*(x+6));
    shifts=align(textured,shifted,4,columns,rows);correct=0;
    for(const auto &s:shifts)if(s.valid && s.x==6 && s.y==5)++correct;
    if(correct<int(shifts.size())/2) {qCritical()<<"translation alignment"<<correct<<shifts.size();return false;}
    // An occluding object must not leave a ghost in the reference background.
    for(int y=64;y<128;++y)for(int x=64;x<128;++x) {
        auto *p=shifted.scanLine(y)+x*3;p[0]=250;p[1]=10;p[2]=20;
    }
    auto occlusion=denoiseBurst({{textured,metadata},{shifted,metadata}});
    double redError=0;int samples=0;
    for(int y=70;y<115;++y)for(int x=70;x<115;++x) {
        auto *p=occlusion.image.constScanLine(y)+3*x;
        redError+=std::abs(int(p[0])-p[1]);++samples;
    }
    if(redError/samples>3) {qCritical()<<"moving-object ghost"<<redError/samples;return false;}
    qInfo()<<"Noise tests: chroma variance ratio"<<after/before<<"preview chroma ratio"<<previewError/before<<"burst luminance error ratio"<<newError/oldError;
    return true;
}
