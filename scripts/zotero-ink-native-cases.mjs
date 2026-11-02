import assert from 'node:assert/strict';
import {join} from 'node:path';

export async function runInkCases({page,editor,annotations,until,refresh,report,out,setMode}){
 const filter=annotations.getByRole('combobox',{name:'筛选Zotero批注类型'}),row=annotations.locator('[data-zotero-list-key="IK000001"]');
 await filter.selectOption('ink');await row.getByRole('img',{name:'Zotero笔迹图形预览'}).waitFor();
 assert.equal(await row.locator('polyline').count(),2);assert.equal(await row.locator('circle').count(),1);
 await row.getByRole('button',{name:/^Zotero ·/}).click();await editor.locator('.pdf-source-highlight').waitFor();
 const canvas=()=>editor.locator('[data-pdf-page="0"] canvas[data-dense-annotations="1"]');
 const pixels=async(turn)=>until(async()=>{
  if(!await canvas().count())return false;
  return canvas().evaluate((c,turn)=>{
   if(c.dataset.annotationPaint!=='complete')return false;
   const dimensions=turn%2?[800,600]:[600,800],project=(x,y)=>[[x,800-y],[y,x],[600-x,y],[800-y,600-x]][turn];
   const red=(x,y)=>{const p=project(x,y),px=Math.round(p[0]/dimensions[0]*c.width),py=Math.round(p[1]/dimensions[1]*c.height),ctx=c.getContext('2d');
    const data=ctx.getImageData(Math.max(0,px-2),Math.max(0,py-2),5,5).data;for(let i=0;i<data.length;i+=4)if(data[i]>220&&data[i+1]>65&&data[i+1]<150&&data[i+2]>65&&data[i+2]<150&&data[i+3]>150)return true;return false;};
   return [[100,550],[200,650],[300,550],[200,600],[120,570],[280,570]].every(([x,y])=>red(x,y))&&!red(400,400);
  },turn);
 },'ink painted pixels at '+turn*90+' degrees');
 await pixels(0);report.checks.push('native HTTP/SQLite preserves multi-stroke ink and single dot; sidebar SVG and page canvas contain real graphics, list click locates original PDF bounds');
 await page.screenshot({path:join(out,'ink-preview-and-page.png')});
 for(let turn=1;turn<4;turn++){await editor.getByRole('button',{name:'旋转PDF视图',exact:true}).click();await pixels(turn);}
 await editor.getByRole('button',{name:'旋转PDF视图',exact:true}).click();await pixels(0);
 for(const name of ['放大PDF','缩小PDF']){await editor.getByRole('button',{name,exact:true}).click();await pixels(0);}
 report.checks.push('actual WebView2 canvas red pixels match known PDF coordinates at0/90/180/270 degrees and after PDF zoom changes; blank control point remains transparent');
 const visible=annotations.getByRole('checkbox',{name:'显示Zotero只读批注图层'});await visible.uncheck();assert.equal(await canvas().count(),0);assert.equal(await row.locator('svg').count(),1);await visible.check();await pixels(0);
 await filter.selectOption('highlight');assert.equal(await canvas().count(),0);await filter.selectOption('ink');await pixels(0);
 report.checks.push('overlay visibility and type filters remove/repaint ink correctly while sidebar previews remain independent of overlay visibility');
 // Click a visible stroke, using the page host's click handler rather than the sidebar.
 await canvas().evaluate(c=>{const b=c.getBoundingClientRect();c.closest('.pdf-page').dispatchEvent(new MouseEvent('click',{bubbles:true,clientX:b.left+b.width/3,clientY:b.top+b.height/4}));});
 await until(()=>row.getAttribute('data-focused').then(v=>v==='true'),'ink graphic focuses matching list row');
 setMode('disabled');await refresh();await annotations.getByRole('alert').waitFor();assert.equal(await row.locator('svg').count(),1);await pixels(0);setMode('inkPreview');
 await annotations.getByRole('button',{name:'重载来源与缓存',exact:true}).click();await until(()=>row.locator('svg').count().then(n=>n===1),'offline cache preview reload');await pixels(0);
 report.checks.push('clicking page ink focuses the original Zotero key; HTTP403 and local-cache reload preserve graphical preview and page drawing');
 for(const width of [760,480,1440]){await page.setViewportSize({width,height:1000});await until(()=>page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),'responsive ink layout');}
 report.checks.push('ink preview remains inside the native workbench at480/760/1440 CSS pixels; physical mixed-monitor DPI and real Zotero UI remain manual');
}
