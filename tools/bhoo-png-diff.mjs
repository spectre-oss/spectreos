// Minimal PNG support for opaque 8-bit RGB/RGBA Chromium screenshots. No image dependencies.
import {inflateSync,deflateSync} from 'node:zlib';
const signature=Buffer.from([137,80,78,71,13,10,26,10]);
function crc32(bytes){let crc=0xffffffff;for(const byte of bytes){crc^=byte;for(let i=0;i<8;i++)crc=(crc>>>1)^((crc&1)?0xedb88320:0);}return (crc^0xffffffff)>>>0;}
function chunk(name,data){const type=Buffer.from(name),length=Buffer.alloc(4),crc=Buffer.alloc(4);length.writeUInt32BE(data.length);crc.writeUInt32BE(crc32(Buffer.concat([type,data])));return Buffer.concat([length,type,data,crc]);}
export function encodePng(width,height,rgba){
  const header=Buffer.alloc(13);header.writeUInt32BE(width);header.writeUInt32BE(height,4);header[8]=8;header[9]=6;
  const stride=width*4,rows=Buffer.alloc((stride+1)*height);
  for(let y=0;y<height;y++)Buffer.from(rgba.buffer,rgba.byteOffset+y*stride,stride).copy(rows,y*(stride+1)+1);
  return Buffer.concat([signature,chunk('IHDR',header),chunk('IDAT',deflateSync(rows)),chunk('IEND',Buffer.alloc(0))]);
}
export function decodePng(png){
  if(!png.subarray(0,8).equals(signature))throw Error('Invalid PNG');
  let width,height,channels;const parts=[];
  for(let at=8;at<png.length;){const length=png.readUInt32BE(at),name=png.toString('ascii',at+4,at+8),data=png.subarray(at+8,at+8+length);at+=length+12;
    if(name==='IHDR'){width=data.readUInt32BE(0);height=data.readUInt32BE(4);if(data[8]!==8||![2,6].includes(data[9])||data[12]!==0)throw Error('Expected non-interlaced 8-bit RGB/RGBA screenshot');channels=data[9]===2?3:4;}
    if(name==='IDAT')parts.push(data);
  }
  const bytes=inflateSync(Buffer.concat(parts)),stride=width*channels,pixels=Buffer.alloc(stride*height);
  const paeth=(a,b,c)=>{const p=a+b-c,pa=Math.abs(p-a),pb=Math.abs(p-b),pc=Math.abs(p-c);return pa<=pb&&pa<=pc?a:pb<=pc?b:c;};
  for(let y=0;y<height;y++){const filter=bytes[y*(stride+1)];if(filter>4)throw Error('Invalid PNG filter');for(let x=0;x<stride;x++){
    const at=y*stride+x,left=x>=channels?pixels[at-channels]:0,up=y?pixels[at-stride]:0,corner=y&&x>=channels?pixels[at-stride-channels]:0;
    pixels[at]=(bytes[y*(stride+1)+1+x]+[0,left,up,Math.floor((left+up)/2),paeth(left,up,corner)][filter])&255;
  }}
  const rgba=Buffer.alloc(width*height*4);for(let i=0;i<width*height;i++){rgba[i*4]=pixels[i*channels];rgba[i*4+1]=pixels[i*channels+1];rgba[i*4+2]=pixels[i*channels+2];rgba[i*4+3]=channels===4?pixels[i*channels+3]:255;}
  return {width,height,rgba};
}
export function comparePng(a,b,threshold=16){
  const left=decodePng(a),right=decodePng(b);if(left.width!==right.width||left.height!==right.height)throw Error('Comparison dimensions differ');
  const heat=Buffer.alloc(left.rgba.length);let changedPixels=0,totalDelta=0;
  for(let at=0;at<heat.length;at+=4){let max=0;for(let c=0;c<3;c++){const delta=Math.abs(left.rgba[at+c]-right.rgba[at+c]);totalDelta+=delta;max=Math.max(max,delta);}
    if(max>threshold){changedPixels++;heat[at]=240;heat[at+1]=Math.max(0,210-max);heat[at+2]=Math.max(0,210-max);}else{heat[at]=heat[at+1]=heat[at+2]=245;}heat[at+3]=255;
  }
  const totalPixels=left.width*left.height;
  return {png:encodePng(left.width,left.height,heat),width:left.width,height:left.height,threshold,changedPixels,totalPixels,changedFraction:changedPixels/totalPixels,meanAbsChannelDelta:totalDelta/(totalPixels*3)};
}
export function cropPng(png,x,y,width,height){
  const source=decodePng(png);x=Math.round(x);y=Math.round(y);width=Math.round(width);height=Math.round(height);
  if(x<0||y<0||x+width>source.width||y+height>source.height)throw Error(`Crop outside screenshot: ${x},${y},${width},${height} in ${source.width}×${source.height}`);
  const rgba=Buffer.alloc(width*height*4);
  for(let row=0;row<height;row++)source.rgba.copy(rgba,row*width*4,((y+row)*source.width+x)*4,((y+row)*source.width+x+width)*4);
  return encodePng(width,height,rgba);
}
