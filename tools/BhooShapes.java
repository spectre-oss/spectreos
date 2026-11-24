/* Shape recipe adaptation: Copyright 2024 The Android Open Source Project.
 * SPDX-License-Identifier: Apache-2.0
 * Source: MaterialShapes.kt at 1448d64974931a04c9fba96ac630cbd6b96cda36.
 * Changes: Java export of only the seven loading shapes and their matched cubics.
 * Uses AndroidX graphics-shapes 1.1.0 at generation time, never at runtime.
 */
import androidx.graphics.shapes.*;
import java.util.*;
import androidx.compose.animation.core.FloatSpringSpec;
public final class BhooShapes {
    static long point(float x,float y){return ((long)Float.floatToRawIntBits(x)<<32)|(Float.floatToRawIntBits(y)&0xffffffffL);}
    static RoundedPolygon transform(RoundedPolygon polygon,float angle,float sy){
        final double a=angle*Math.PI/180;
        return polygon.transformed((x,y)->point((float)(x*Math.cos(a)-y*sy*Math.sin(a)),(float)(x*Math.sin(a)+y*sy*Math.cos(a))));
    }
    static RoundedPolygon custom(float[][] points,int reps,boolean mirror){
        List<Float> vertices=new ArrayList<>();List<CornerRounding> rounding=new ArrayList<>();
        int actual=mirror?reps*2:reps;
        for(int rep=0;rep<actual;rep++){
            for(int j=0;j<points.length;j++){
                int i=mirror&&rep%2!=0?points.length-1-j:j;
                if(mirror&&i==0&&rep%2!=0)continue;
                float[] p=points[i];double x=p[0]-.5,y=p[1]-.5;
                double start=Math.atan2(points[0][1]-.5,points[0][0]-.5);
                double a=Math.atan2(y,x),distance=Math.hypot(x,y),section=2*Math.PI/actual;
                double angle=section*rep+(mirror&&rep%2!=0?section-a+2*start:a);
                vertices.add((float)(Math.cos(angle)*distance+.5));vertices.add((float)(Math.sin(angle)*distance+.5));
                rounding.add(new CornerRounding(p.length>2?p[2]:0,p.length>3?p[3]:0));
            }
        }
        float[] v=new float[vertices.size()];for(int i=0;i<v.length;i++)v[i]=vertices.get(i);
        return RoundedPolygonKt.RoundedPolygon(v,CornerRounding.Unrounded,rounding,.5f,.5f).normalized();
    }
    static RoundedPolygon[] shapes(){return new RoundedPolygon[]{
        custom(new float[][]{{.193f,.277f,.053f},{.176f,.055f,.053f}},10,false),
        transform(ShapesKt.star(RoundedPolygon.Companion,9,1f,.8f,new CornerRounding(.5f,0)), -90,1).normalized(),
        custom(new float[][]{{.5f,-.009f,.172f},{1.030f,.365f,.164f},{.828f,.970f,.169f}},1,true),
        custom(new float[][]{{.961f,.039f,.426f},{1.001f,.428f},{1f,.609f,1f}},2,true),
        ShapesKt.star(RoundedPolygon.Companion,8,1f,.8f,new CornerRounding(.15f,0)).normalized(),
        custom(new float[][]{{1.237f,1.236f,.258f},{.5f,.918f,.233f}},4,false),
        transform(ShapesKt.circle(RoundedPolygon.Companion),-45,.64f).normalized()
    };}
    static void cubic(Cubic c){float[] p=c.getPoints$graphics_shapes();System.out.print("[");for(int i=0;i<8;i++){if(i>0)System.out.print(",");System.out.print(Float.toString(p[i]));}System.out.print("]");}
    public static void main(String[] args){
        RoundedPolygon[] shapes=shapes();float scale=1;
        for(RoundedPolygon shape:shapes){float[] b=shape.calculateBounds();float[] max=shape.calculateMaxBounds(new float[4]);scale=Math.min(scale,Math.max((b[2]-b[0])/(max[2]-max[0]),(b[3]-b[1])/(max[3]-max[1])));}
        System.out.print("{\"scale\":"+scale+",\"spring_seconds\":"+new FloatSpringSpec(.6f,200f,.1f).getDurationNanos(0f,1f,0f)/1e9+",\"pairs\":[");
        for(int i=0;i<8;i++){if(i>0)System.out.print(",");Morph morph=i<7?new Morph(shapes[i],shapes[(i+1)%7]):new Morph(transform(ShapesKt.circle(RoundedPolygon.Companion,10),18,1).normalized(),shapes[0]);System.out.print("[");boolean first=true;for(var pair:morph.getMorphMatch()){if(!first)System.out.print(",");first=false;System.out.print("[");cubic(pair.getFirst());System.out.print(",");cubic(pair.getSecond());System.out.print("]");}System.out.print("]");}
        System.out.println("]}");
    }
}
