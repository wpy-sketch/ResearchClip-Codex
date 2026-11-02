import {describe,it,expect,vi} from 'vitest';
import {renderToStaticMarkup} from 'react-dom/server';
import {InkPreview} from './InkPreview';
import {AnnotationCoordinateAdapter,type AnnotationViewport} from './AnnotationCoordinateAdapter';
import {drawAnnotation} from './drawing';
import type {ZoteroAnnotation} from './model';

const ink:ZoteroAnnotation={identity:{source:{instanceId:'a'.repeat(32),libraryType:'user',libraryId:0,itemKey:'PARENT01',attachmentKey:'ATTACH01'},annotationKey:'INK00001'},version:1,annotationType:'ink',color:'#ff6666',text:'',comment:'',pageLabel:'1',position:{pageIndex:0,rects:[[9,19,61,81]],nextPageRects:[],paths:[[10,20,30,40,60,20],[40,80]],width:2},unsupportedReason:null};
const viewport:AnnotationViewport={width:600,height:800,convertToViewportPoint:(x,y)=>[x,800-y]};
function context(){return {save:vi.fn(),restore:vi.fn(),setLineDash:vi.fn(),beginPath:vi.fn(),moveTo:vi.fn(),lineTo:vi.fn(),stroke:vi.fn()} as unknown as CanvasRenderingContext2D;}

describe('Zotero ink graphical previews and PDF drawing',()=>{
 it('renders separate strokes and a round dot with PDF Y direction, padding and original color/width',()=>{
  const before=JSON.stringify(ink),html=renderToStaticMarkup(<InkPreview annotation={ink}/>);
  expect(html).toContain('aria-label="Zotero笔迹图形预览"');expect(html).toContain('viewBox="0 0 56 66"');
  expect(html).toContain('points="3,63 23,43 53,63"');expect(html).toContain('stroke="#ff6666" stroke-width="2"');
  expect(html).toContain('<circle');expect(html).toContain('cx="33" cy="3" r="1"');expect(JSON.stringify(ink)).toBe(before);
 });
 it('gives a single point a nonempty viewBox without inventing a line',()=>{
  const html=renderToStaticMarkup(<InkPreview annotation={{...ink,position:{...ink.position!,paths:[[0,0]],width:4}}}/>);
  expect(html).toContain('viewBox="0 0 12 12"');expect(html).toContain('r="2"');expect(html).not.toContain('<polyline');
 });
 it('renders only ink and explicitly reports missing or invalid geometry without unsafe SVG',()=>{
  expect(renderToStaticMarkup(<InkPreview annotation={{...ink,annotationType:'note'}}/>)).toBe('');
  for(const annotation of [{...ink,position:null},{...ink,color:'url(javascript:bad)'},{...ink,position:{...ink.position!,paths:[[NaN,1]]}},{...ink,position:{...ink.position!,paths:[[1,2,3]]}},{...ink,position:{...ink.position!,width:0}}]){
   const html=renderToStaticMarkup(<InkPreview annotation={annotation}/>);expect(html).toContain('笔迹图形不可用');expect(html).not.toContain('<svg');
  }
 });
 const turns=[{width:1200,height:1600,convertToViewportPoint:(x:number,y:number)=>[2*x-80,1700-2*y]},
  {width:1600,height:1200,convertToViewportPoint:(x:number,y:number)=>[2*y-100,2*x-80]},
  {width:1200,height:1600,convertToViewportPoint:(x:number,y:number)=>[1280-2*x,2*y-100]},
  {width:1600,height:1200,convertToViewportPoint:(x:number,y:number)=>[1700-2*y,1280-2*x]}];
 for(const [i,v] of turns.entries())it('projects crop offsets, UserUnit2 and physical line width at '+i*90+' degrees',()=>{
  const a={...ink,position:{...ink.position!,paths:[[140,650,340,700]]}},s=AnnotationCoordinateAdapter.strokes(a,0,v)[0];
  expect(s.points).toEqual([[200,400,600,300],[1200,200,1300,600],[1000,1200,600,1300],[400,1000,300,600]][i]);expect(s.width).toBe(4);
  expect(AnnotationCoordinateAdapter.strokes(a,1,v)).toEqual([]);
 });
 it('refuses invalid viewports, incomplete points and collapsed width transforms',()=>{
  for(const v of [{...viewport,width:0},{...viewport,height:NaN},{...viewport,convertToViewportPoint:()=>[1]},{...viewport,convertToViewportPoint:()=>[1,Infinity]},{...viewport,convertToViewportPoint:()=>[1,1]}])expect(()=>AnnotationCoordinateAdapter.strokes(ink,0,v)).toThrow();
 });
 it('scales the page drawing with bitmap size while retaining strokes and round caps',()=>{
  const ctx=context();drawAnnotation(ctx,ink,0,viewport,1200,1600,2);
  expect(ctx.moveTo).toHaveBeenNthCalledWith(1,20,1560);expect(ctx.lineTo).toHaveBeenCalledWith(60,1520);expect(ctx.stroke).toHaveBeenCalledTimes(2);
  expect(ctx.strokeStyle).toBe('#ff6666');expect(ctx.lineWidth).toBe(4);expect(ctx.lineCap).toBe('round');expect(ctx.restore).toHaveBeenCalledOnce();
 });
 it('restores canvas state on invalid geometry so the next annotation can still render',()=>{
  const ctx=context();expect(()=>drawAnnotation(ctx,{...ink,position:{...ink.position!,paths:[[1,2,3]]}},0,viewport,600,800,1)).toThrow();
  expect(ctx.restore).toHaveBeenCalledOnce();drawAnnotation(ctx,ink,0,viewport,600,800,1);expect(ctx.stroke).toHaveBeenCalledTimes(2);expect(ctx.restore).toHaveBeenCalledTimes(2);
 });
});
