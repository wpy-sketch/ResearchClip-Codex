/** Map existing features to existing configuration sections, without a second service. */
export function configurationSection(id:string):string {
 if(id.includes('zotero')||id.startsWith('pdf.'))return 'zotero-setup-title';
 if(id.includes('clipper')||id==='plugin.browser-clipper')return 'clipper-title';
 if(id.includes('webdav'))return 'webdav-title';
 if(id.includes('semantic'))return 'semantic-settings-title';
 if(id.includes('qwen')||id.includes('similarity'))return 'ai-settings-title';
 if(id.includes('latex')||id.includes('python')||id.includes('math'))return 'math-ocr-settings-title';
 if(id.includes('browser'))return 'browser-settings-title';
 if(id.includes('command')||id.includes('workflow'))return 'commands-settings-title';
 if(id.startsWith('capture.'))return 'shortcut-title';
 return 'feature-packages-title';
}
