import {useEffect,useRef,useState} from 'react';
import type {AnnotationViewport} from './AnnotationCoordinateAdapter';
import {drawAnnotation} from './drawing';
import type {ZoteroAnnotation} from './model';
/** Dense pages use a bounded bitmap, drawing every supported region in cancellable frames. */
export function DenseAnnotationCanvas({items,index,viewport}:{items:readonly ZoteroAnnotation[];index:number;viewport:AnnotationViewport}){
 const ref=useRef<HTMLCanvasElement>(null),[error,setError]=useState('');
 useEffect(()=>{const canvas=ref.current;if(!canvas)return;let frame=0,active=true;const ctx=canvas.getContext('2d');if(!ctx)return;
  const paint=()=>{cancelAnimationFrame(frame);const rect=canvas.getBoundingClientRect(),scale=Math.min(window.devicePixelRatio||1,2,Math.sqrt(4_000_000/Math.max(1,rect.width*rect.height)));canvas.width=Math.max(1,Math.round(rect.width*scale));canvas.height=Math.max(1,Math.round(rect.height*scale));const w=canvas.width,h=canvas.height;let cursor=0;
   const chunk=()=>{if(!active)return;const end=Math.min(items.length,cursor+80),deadline=performance.now()+8;try{for(;cursor<end&&performance.now()<deadline;cursor++){drawAnnotation(ctx,items[cursor],index,viewport,w,h,scale);}}catch{setError('部分批注坐标无法绘制；列表和原始缓存保留。');return;}if(cursor<items.length)frame=requestAnimationFrame(chunk);};frame=requestAnimationFrame(chunk);
  };const observer=new ResizeObserver(paint);observer.observe(canvas);paint();return()=>{active=false;cancelAnimationFrame(frame);observer.disconnect();};
 },[items,index,viewport]);
 return <><canvas ref={ref} className="zotero-annotation-canvas" aria-label={'密集页面全部标准批注图层（'+items.length+'条）'} data-dense-annotations={items.length}/>{error&&<span role="status" className="zotero-overlay-warning">{error}</span>}</>;
}
