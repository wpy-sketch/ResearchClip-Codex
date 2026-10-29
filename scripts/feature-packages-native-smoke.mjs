// Own isolated Tauri process/WebView2/Rust IPC; no real Zotero or user workspace.
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {mkdir,copyFile,writeFile,readFile} from 'node:fs/promises';
import {resolve,join} from 'node:path';
import {setTimeout as delay} from 'node:timers/promises';
import {chromium} from 'playwright-core';
const root=resolve('.'),run=process.argv[2];assert.match(run??'',/^[a-z0-9-]{1,40}$/);
const out=join(root,'artifacts','feature-packages-native-'+run),settings=join(out,'settings'),port=9416;
await mkdir(out);await mkdir(settings);
const binary=join(out,'researchclip-step1-test.exe');
await copyFile(join(root,'src-tauri/target/debug/researchclip.exe'),binary,1);
await writeFile(join(settings,'settings.json'),JSON.stringify({schemaVersion:1,theme:'dark',viewMode:'grid',fontFamily:'system',fontSize:16,captureShortcut:'Ctrl+Alt+Shift+F7',fullscreenShortcut:'Ctrl+Alt+Shift+F8',saveDirectory:null,closeToTray:false}));
const report={mode:'Windows Tauri debug/custom-protocol, real WebView2/Rust IPC, isolated local data',checks:[],errors:[],unverified:['Physical DPI/HDR/mixed monitors','Actual browser extension and native host installation','Actual Zotero XPI/library/API/write permissions','External WebDAV provider and downloaded model execution']};
let app,browser,page;
async function until(fn,name,ms=30000){const start=Date.now();let error;while(Date.now()-start<ms){try{const value=await fn();if(value)return value;}catch(e){error=e;}await delay(100);}throw Error(name+': '+(error?.message??'timeout'));}
const call=(name,args)=>page.evaluate(({name,args})=>window.__TAURI_INTERNALS__.invoke(name,args),{name,args});
try{
 assert.equal(await fetch(`http://127.0.0.1:${port}/json/version`).then(()=>true).catch(()=>false),false);
 app=spawn(binary,[],{cwd:root,windowsHide:true,stdio:'ignore',env:{...process.env,RESEARCHCLIP_TEST_SETTINGS_DIR:settings,WEBVIEW2_USER_DATA_FOLDER:join(out,'webview'),WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:`--remote-debugging-port=${port} --remote-debugging-address=127.0.0.1`}});
 await until(()=>fetch(`http://127.0.0.1:${port}/json/version`).then(r=>r.ok).catch(()=>false),'native app');
 browser=await chromium.connectOverCDP(`http://127.0.0.1:${port}`);
 page=await until(()=>browser.contexts().flatMap(c=>c.pages()).find(p=>p.url().includes('tauri')),'main WebView');page.on('pageerror',e=>report.errors.push(e.message));
 assert.equal((await call('library_status')).schemaVersion,21);report.checks.push('actual isolated SQLite schema21 initialized without a new Step1 migration');
 await page.getByRole('button',{name:'设置',exact:true}).click();const section=page.locator('.feature-packages');
 await section.locator('.feature-package-catalog > summary').click();
 assert.equal(await section.locator('[data-feature-id]').count(),48);
 assert.equal(await section.getByRole('combobox',{name:'筛选组件类型'}).locator('option').count(),7);report.checks.push('actual embedded frontend renders 48 definitions and all six package classes');
 const plugin=section.locator('[data-feature-id="plugin.browser-clipper"]');await plugin.locator('summary').click();assert.match(await plugin.innerText(),/未知/);
 const updater=section.locator('[data-feature-id="app.auto-updater"]');await updater.locator('summary').click();assert.match(await updater.innerText(),/NOT_CHECKED|APP_UPDATE_REQUIRED/);
 assert.equal(await section.getByRole('button',{name:'检查本机组件状态',exact:true}).isEnabled(),true);report.checks.push('external installation remains UNKNOWN and auto updater is explicitly unimplemented before checking');
 await section.getByRole('button',{name:'检查本机组件状态',exact:true}).click();await until(()=>section.getByText('已读取本机状态；安装、启用、依赖与验收分别列出。',{exact:true}).isVisible(),'status aggregation');
 assert.match(await plugin.innerText(),/INSTALLATION_UNKNOWN/);assert.match(await updater.innerText(),/APP_UPDATE_REQUIRED/);
 const clipper=section.locator('[data-feature-id="clipper.bridge"]');await clipper.locator('summary').click();
 assert.equal((await call('clipper_status')).enabled,false);await clipper.getByRole('button',{name:'启用本机桥',exact:true}).click();await until(()=>clipper.getByRole('button',{name:'停用本机桥',exact:true}).isEnabled(),'enable clipper');assert.equal((await call('clipper_status')).enabled,true);
 await clipper.getByRole('button',{name:'停用本机桥',exact:true}).click();await until(()=>clipper.getByRole('button',{name:'启用本机桥',exact:true}).isEnabled(),'disable clipper');assert.equal((await call('clipper_status')).enabled,false);report.checks.push('catalog controls original clipper service and confirms real native enabled/disabled state');
 const connector=section.locator('[data-feature-id="zotero.connector-bridge"]');await connector.locator('summary').click();await connector.getByRole('button',{name:'启用本机桥',exact:true}).click();await until(()=>connector.getByRole('button',{name:'停用本机桥',exact:true}).isEnabled(),'enable connector');assert.equal((await call('zotero_connector',{request:{operation:'status'}})).status.enabled,true);
 await connector.getByRole('button',{name:'停用本机桥',exact:true}).click();await until(()=>connector.getByRole('button',{name:'启用本机桥',exact:true}).isEnabled(),'disable connector');assert.equal((await call('zotero_connector',{request:{operation:'status'}})).status.enabled,false);report.checks.push('catalog controls original opt-in Connector service without pairing or touching Zotero');
 const filter=section.getByRole('combobox',{name:'筛选组件类型'});await filter.selectOption('MODEL_PACKAGE');assert.equal(await section.locator('[data-feature-id]').count(),3);await filter.selectOption('');const search=section.getByRole('textbox',{name:'查找功能ID或名称'});await search.fill('pdf.reader');assert.equal(await section.locator('[data-feature-id]').count(),1);await search.fill('');report.checks.push('class filtering and stable feature ID search work');
 for(const width of [1440,760,480]){await page.setViewportSize({width,height:920});await delay(100);assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1));}
 await page.setViewportSize({width:1440,height:920});for(const zoom of [.75,1,1.25,1.5,2]){await page.evaluate(z=>document.documentElement.style.zoom=String(z),zoom);await delay(100);assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1));}await page.evaluate(()=>document.documentElement.style.zoom='');report.checks.push('native WebView contained at 480/760/1440 widths and CSS scale75–200%; not physical Windows DPI acceptance');
 await page.screenshot({path:join(out,'feature-catalog.png')});
 await page.getByRole('button',{name:'新建空白 Markdown',exact:true}).click();const editor=page.locator('textarea.markdown-source');await editor.fill('# Step1 草稿\n\n$x^2$');await page.getByRole('button',{name:'截图素材',exact:true}).click();assert.equal(await editor.inputValue(),'# Step1 草稿\n\n$x^2$');await page.locator('[data-markdown-panel]').getByRole('button',{name:'关闭文档',exact:true}).click();await page.getByRole('button',{name:'取消',exact:true}).click();assert.equal(await editor.inputValue(),'# Step1 草稿\n\n$x^2$');await page.locator('[data-markdown-panel]').getByRole('button',{name:'关闭文档',exact:true}).click();await page.getByRole('button',{name:'放弃修改',exact:true}).click();await until(()=>page.locator('[data-markdown-panel]').count().then(n=>n===0),'no empty split');report.checks.push('settings integration preserves Markdown draft and close confirmation; no empty Markdown panel remains');
 report.settingsSchema=JSON.parse(await readFile(join(settings,'settings.json'),'utf8')).schemaVersion;assert.deepEqual(report.errors,[]);report.result='PASS';
}catch(e){report.result='FAIL';report.failure=e.stack;process.exitCode=1;if(page&&!page.isClosed())await page.screenshot({path:join(out,'failure.png')}).catch(()=>{});}
finally{if(page&&!page.isClosed()){try{await call('clipper_connection',{enabled:false});await call('zotero_connector',{request:{operation:'disable'}});await call('desktop_action',{action:'exit'});await until(()=>app.exitCode!==null,'clean exit',10000);}catch(e){report.errors.push('Cleanup: '+e.message);report.result='FAIL';process.exitCode=1;}}await browser?.close().catch(()=>{});if(app&&app.exitCode===null)app.kill();await writeFile(join(out,'report.json'),JSON.stringify(report,null,2));console.log(JSON.stringify({result:report.result,checks:report.checks.length,failure:report.failure??null,out}));}
