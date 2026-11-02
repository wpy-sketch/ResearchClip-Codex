import {validPositionExtras,type ZoteroAnnotation} from './model';

/** Read-only SVG built from validated numbers, never Zotero-provided markup. */
export function InkPreview({annotation}:{annotation:ZoteroAnnotation}) {
 const p=annotation.position;
 if(annotation.annotationType!=='ink')return null;
 if(!p||!validPositionExtras(p,'ink')||!/^#[a-f0-9]{6}$/.test(annotation.color))return <span className="zotero-ink-unavailable">笔迹图形不可用，请重新读取Zotero批注。</span>;
 const paths=p.paths!,width=p.width!;
 let minX=Infinity,minY=Infinity,maxX=-Infinity,maxY=-Infinity;
 for(const path of paths)for(let i=0;i<path.length;i+=2){minX=Math.min(minX,path[i]);maxX=Math.max(maxX,path[i]);minY=Math.min(minY,path[i+1]);maxY=Math.max(maxY,path[i+1]);}
 // PDF coordinates run upwards; include cap radius and padding, including single dots.
 const padding=width*1.5,w=maxX-minX+padding*2,h=maxY-minY+padding*2;
 return <svg className="zotero-ink-preview" role="img" aria-label="Zotero笔迹图形预览" viewBox={`0 0 ${w} ${h}`} preserveAspectRatio="xMidYMid meet">
  {paths.map((path,index)=>path.length===2?<circle key={index} cx={path[0]-minX+padding} cy={maxY-path[1]+padding} r={width/2} fill={annotation.color}/>:<polyline key={index} points={Array.from({length:path.length/2},(_,i)=>`${path[i*2]-minX+padding},${maxY-path[i*2+1]+padding}`).join(' ')} fill="none" stroke={annotation.color} strokeWidth={width} strokeLinecap="round" strokeLinejoin="round"/>)}
 </svg>;
}
