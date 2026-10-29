import type {ZoteroAnnotation} from './model';
import {AnnotationCoordinateAdapter,type AnnotationViewport} from './AnnotationCoordinateAdapter';
/** A visual projection of a Zotero standard annotation; never a private handwriting record. */
export function drawAnnotation(ctx:CanvasRenderingContext2D,a:ZoteroAnnotation,index:number,v:AnnotationViewport,w:number,h:number,density:number){
 const sx=w/v.width,sy=h/v.height;ctx.save();ctx.strokeStyle=a.color;ctx.fillStyle=a.color;ctx.globalAlpha=a.annotationType==='highlight'?.28:1;ctx.lineWidth=Math.max(1,2*density);ctx.setLineDash(a.annotationType==='note'?[4*density,3*density]:[]);
 if(a.annotationType==='ink'){
  ctx.lineCap='round';ctx.lineJoin='round';for(const stroke of AnnotationCoordinateAdapter.strokes(a,index,v)){ctx.lineWidth=Math.max(.5,stroke.width*(sx+sy)/2);ctx.beginPath();for(let i=0;i<stroke.points.length;i+=2){const x=stroke.points[i]*sx,y=stroke.points[i+1]*sy;if(i===0)ctx.moveTo(x,y);else ctx.lineTo(x,y);}if(stroke.points.length===2){ctx.lineTo(stroke.points[0]*sx+.001,stroke.points[1]*sy);}ctx.stroke();}
 }else if(a.annotationType==='text'&&a.position?.pageIndex===index){
  const p=a.position,r=p.rects[0],origin=v.convertToViewportPoint(0,0),x=v.convertToViewportPoint(1,0),y=v.convertToViewportPoint(0,1);ctx.transform((x[0]-origin[0])*sx,(x[1]-origin[1])*sy,(y[0]-origin[0])*sx,(y[1]-origin[1])*sy,origin[0]*sx,origin[1]*sy);ctx.translate(r[0],r[3]);ctx.rotate(-(p.rotation??0)*Math.PI/180);ctx.scale(1,-1);const font=p.fontSize??12;ctx.font=`${font}px sans-serif`;ctx.textBaseline='top';let row=0,line='';const width=Math.max(1,r[2]-r[0]);for(const char of (a.comment||a.text).slice(0,8192)+((a.comment||a.text).length>8192?'…完整文本见批注列表':'')){if(char==='\n'||ctx.measureText(line+char).width>width&&line){ctx.fillText(line,0,row++*font*1.2);line='';if(row>=200)break;}if(char!=='\n')line+=char;}if(row<200&&line)ctx.fillText(line,0,row*font*1.2);
 }else for(const r of AnnotationCoordinateAdapter.regions(a,index,v)){if(r.line){ctx.beginPath();ctx.moveTo(r.line.x1*w,r.line.y1*h);ctx.lineTo(r.line.x2*w,r.line.y2*h);ctx.stroke();}else if(a.annotationType==='highlight')ctx.fillRect(r.box.x*w,r.box.y*h,r.box.width*w,r.box.height*h);else ctx.strokeRect(r.box.x*w,r.box.y*h,r.box.width*w,r.box.height*h);}
 ctx.restore();
}
