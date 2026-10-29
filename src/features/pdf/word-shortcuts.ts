export const wordShortcutKey='researchclip.pdf.word-shortcuts.v1';
export const wordActions=['highlight','underline','copy','translate'] as const;
export type WordAction=typeof wordActions[number];
export const wordChords=['Ctrl+Alt+H','Ctrl+Alt+U','Ctrl+Alt+C','Ctrl+Alt+T','Ctrl+Shift+H','Ctrl+Shift+U','Ctrl+Shift+C','Ctrl+Shift+T'] as const;
export type WordShortcuts=Record<WordAction,string>;
export const defaultWordShortcuts:WordShortcuts={highlight:wordChords[0],underline:wordChords[1],copy:wordChords[2],translate:wordChords[3]};
export function canonicalChord(chord:string){return chord.replace(/\s/g,'').toLowerCase().replace(/commandorcontrol|control/g,'ctrl').split('+').sort().join('+');}
export function validateWordShortcuts(value:unknown,reserved:readonly string[]):WordShortcuts{const v=value as WordShortcuts|null;if(!v||Object.keys(v).length!==4||wordActions.some(a=>!wordChords.includes(v[a] as typeof wordChords[number])))throw Error('PDF快捷键配置无效，原设置保留。');const chords=wordActions.map(a=>canonicalChord(v[a]));if(new Set(chords).size!==4||chords.some(c=>reserved.some(r=>canonicalChord(r)===c)))throw Error('PDF快捷键重复或与截图快捷键冲突。');return v;}
export function wordShortcut(event:Pick<KeyboardEvent,'ctrlKey'|'shiftKey'|'altKey'|'metaKey'|'key'|'isComposing'|'repeat'>,shortcuts:WordShortcuts,reserved:readonly string[]):WordAction|null{if(event.isComposing||event.metaKey||!event.ctrlKey||event.repeat)return null;const chord=canonicalChord(['Ctrl',...(event.altKey?['Alt']:[]),...(event.shiftKey?['Shift']:[]),event.key].join('+'));if(reserved.some(r=>canonicalChord(r)===chord))return null;return wordActions.find(a=>canonicalChord(shortcuts[a])===chord)??null;}
