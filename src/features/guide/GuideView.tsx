import bingNativeImage from '../../../docs/images/v510-bing-native.png';
import componentInstallImage from '../../../docs/images/v510-component-install.svg';
import sixTypeFlowImage from '../../../docs/images/v440-six-type-flow.svg';
import browserBookmarksImage from '../../../docs/images/v501-browser-bookmarks.svg';
import browserGuideImage from '../../../docs/images/v500-browser-guide.svg';
import packageCenterImage from '../../../docs/images/step2-package-center.png';
import largeAnnotationBrowserImage from '../../../docs/images/v440-annotations-browser.png';
import lastAnnotationSearchImage from '../../../docs/images/v440-annotation-last-search.png';
import largeAnnotationFlowImage from '../../../docs/images/v440-annotation-flow.svg';
import connectorSettingsImage from '../../../docs/images/v428-connector-settings.png';
import connectorFlowImage from '../../../docs/images/v428-connector-flow.svg';
import syncConflictImage from '../../../docs/images/v427-zotero-sync-conflict.png';
import syncHistoryImage from '../../../docs/images/v427-zotero-sync-history.png';
import syncStatusImage from '../../../docs/images/v427-zotero-sync-status.png';
import writeVerifiedImage from '../../../docs/images/v426-zotero-write-verified.png';
import writeDeniedImage from '../../../docs/images/v426-zotero-write-denied.png';
import writePendingImage from '../../../docs/images/v426-zotero-write-pending.png';
import annotationListImage from '../../../docs/images/v425-zotero-annotations.png';
import annotationCropImage from '../../../docs/images/v425-zotero-annotation-crop.png';
import annotationOfflineImage from '../../../docs/images/v425-zotero-annotations-offline.png';
import bidirectionalImage from '../../../docs/images/v424-bidirectional-notes.png';
import textSourceImage from '../../../docs/images/v424-text-source.png';
import excerptSelectionImage from '../../../docs/images/v423-excerpt-selection.png';
import excerptCaptureImage from '../../../docs/images/v423-excerpt-capture.png';
import excerptMarkdownImage from '../../../docs/images/v423-excerpt-markdown.png';
import excerptReturnImage from '../../../docs/images/v423-excerpt-return.png';
import mixedPagesImage from '../../../docs/images/v422-mixed-pages.png';
import mixedNarrowImage from '../../../docs/images/v422-mixed-narrow.png';
import qualityImage from '../../../docs/images/v421-pdf-quality.png';
import narrowPdfImage from '../../../docs/images/v421-pdf-narrow.png';
import zoteroSetupImage from '../../../docs/images/v421-zotero-setup.png';
import zoteroPreview421Image from '../../../docs/images/v421-zotero-preview.png';
import {useUiStore} from '../../app/ui-store';
import zoteroPreviewImage from '../../../docs/images/v420-zotero-preview.png';
import zoteroSavedImage from '../../../docs/images/v420-zotero-saved.png';
import zoteroOfflineImage from '../../../docs/images/v420-zotero-offline.png';
import pdfSpreadImage from '../../../docs/images/v419-pdf-spread.png';
import pdfSearchImage from '../../../docs/images/v419-pdf-search.png';
import pdfMarkdownImage from '../../../docs/images/v419-pdf-markdown.png';
import citationLibraryImage from '../../../docs/images/v418-citation-library.png';
import citationCompletionImage from '../../../docs/images/v418-citation-completion.png';
import citationBibliographyImage from '../../../docs/images/v418-citation-bibliography.png';
import documentDirectoryImage from '../../../docs/images/v417-document-directory.png';
import documentHistoryImage from '../../../docs/images/v417-document-history.png';
import markdownPaperImage from '../../../docs/images/v416-markdown-paper.png';
import markdownTableImage from '../../../docs/images/v415-markdown-table.png';
import markdownVisualImage from '../../../docs/images/v414-markdown-visual.png';
import markdownToolbarImage from '../../../docs/images/v413-markdown-toolbar.png';
import commandTasksImage from '../../../docs/images/v412-command-tasks.png';
import commandPaletteImage from '../../../docs/images/v318-command-palette.png';
import anchorLinkImage from '../../../docs/images/v317-anchor-link.png';
import graphImage from '../../../docs/images/v315-graph.png';
import backlinksImage from '../../../docs/images/v315-backlinks.png';
import referencesImage from '../../../docs/images/v313-references-light.png';
import similarityScopesImage from '../../../docs/images/v313-similarity-scopes.png';
import similarityImage from '../../../docs/images/v312-similarity.png';
import similarityFlowImage from '../../../docs/images/v312-similarity-flow.svg';
import windowCaptureImage from '../../../docs/images/v301-window-capture.svg';
import syncSettingsImage from '../../../docs/images/v310-sync-settings.png';
import syncActionsImage from '../../../docs/images/v310-sync-actions.png';
import syncFlowImage from '../../../docs/images/v300-sync-flow.svg';
import guideDarkImage from '../../../docs/images/v316-guide-dark.png';
import guideLightImage from '../../../docs/images/v316-guide-light.png';
import revisionsImage from '../../../docs/images/v105-revisions.svg';
import mathOcrInputImage from '../../../docs/images/v104-math-ocr-input.png';
import mathOcrWorkflowImage from '../../../docs/images/v104-math-ocr-workflow.svg';
import ocrIndexImage from '../../../docs/images/v103-ocr-index.svg';
import {useEffect,useMemo,useRef,useState,useCallback} from 'react';
import {ChevronRight,Search,BookOpen} from 'lucide-react';
import text from '../../../docs/USER_GUIDE.md?raw';
import {MarkdownPreview} from '../document/MarkdownPreview';
import {Modal} from '../../shared/ui/Modal';
import {guideReadingRoutes,guideTopicGroups as groups,guideSections,searchGuide,guideOffsets,visibleGuideLocation,guideExamples,type GuideSection} from './navigation';
import projectsImage from '../../../docs/images/v021-projects.png';
import mathImage from '../../../docs/images/v021-math.png';
import scrollingImage from '../../../docs/images/v101-scrolling.png';
import clipperSettingsImage from '../../../docs/images/v102-clipper-settings.png';
import clipperExtensionImage from '../../../docs/images/v102-clipper-extension.png';
import captureImage from '../../../docs/images/v100-capture.png';
import library_gridImage from '../../../docs/images/v100-library-grid.png';
import library_listImage from '../../../docs/images/v100-library-list.png';
import editorImage from '../../../docs/images/v100-editor.png';
import cardImage from '../../../docs/images/v100-card.png';
import compareImage from '../../../docs/images/v100-compare.png';
import markdownImage from '../../../docs/images/v100-markdown.png';
import settingsImage from '../../../docs/images/v100-settings.png';
const guideImages={'images/v510-bing-native.png':bingNativeImage,'images/v510-component-install.svg':componentInstallImage,'images/v501-browser-bookmarks.svg':browserBookmarksImage,'images/v500-browser-guide.svg':browserGuideImage,'images/v440-six-type-flow.svg':sixTypeFlowImage,'images/step2-package-center.png':packageCenterImage,'images/v440-annotations-browser.png':largeAnnotationBrowserImage,'images/v440-annotation-last-search.png':lastAnnotationSearchImage,'images/v440-annotation-flow.svg':largeAnnotationFlowImage,'images/v428-connector-settings.png':connectorSettingsImage,'images/v428-connector-flow.svg':connectorFlowImage,'images/v427-zotero-sync-conflict.png':syncConflictImage,'images/v427-zotero-sync-history.png':syncHistoryImage,'images/v427-zotero-sync-status.png':syncStatusImage,'images/v426-zotero-write-verified.png':writeVerifiedImage,'images/v426-zotero-write-denied.png':writeDeniedImage,'images/v426-zotero-write-pending.png':writePendingImage,'images/v425-zotero-annotations.png':annotationListImage,'images/v425-zotero-annotation-crop.png':annotationCropImage,'images/v425-zotero-annotations-offline.png':annotationOfflineImage,'images/v424-bidirectional-notes.png':bidirectionalImage,'images/v424-text-source.png':textSourceImage,'images/v423-excerpt-selection.png':excerptSelectionImage,'images/v423-excerpt-capture.png':excerptCaptureImage,'images/v423-excerpt-markdown.png':excerptMarkdownImage,'images/v423-excerpt-return.png':excerptReturnImage,'images/v422-mixed-pages.png':mixedPagesImage,'images/v422-mixed-narrow.png':mixedNarrowImage,'images/v421-pdf-quality.png':qualityImage,'images/v421-pdf-narrow.png':narrowPdfImage,'images/v421-zotero-setup.png':zoteroSetupImage,'images/v421-zotero-preview.png':zoteroPreview421Image,'images/v420-zotero-preview.png':zoteroPreviewImage,'images/v420-zotero-saved.png':zoteroSavedImage,'images/v420-zotero-offline.png':zoteroOfflineImage,'images/v419-pdf-spread.png':pdfSpreadImage,'images/v419-pdf-search.png':pdfSearchImage,'images/v419-pdf-markdown.png':pdfMarkdownImage,'images/v418-citation-library.png':citationLibraryImage,'images/v418-citation-completion.png':citationCompletionImage,'images/v418-citation-bibliography.png':citationBibliographyImage,'images/v417-document-directory.png':documentDirectoryImage,'images/v417-document-history.png':documentHistoryImage,'images/v416-markdown-paper.png':markdownPaperImage,'images/v415-markdown-table.png':markdownTableImage,'images/v414-markdown-visual.png':markdownVisualImage,'images/v413-markdown-toolbar.png':markdownToolbarImage,'images/v412-command-tasks.png':commandTasksImage,'images/v318-command-palette.png':commandPaletteImage,'images/v317-anchor-link.png':anchorLinkImage,'images/v315-graph.png':graphImage,'images/v315-backlinks.png':backlinksImage,'images/v313-references-light.png':referencesImage,'images/v313-similarity-scopes.png':similarityScopesImage,'images/v312-similarity.png':similarityImage,'images/v312-similarity-flow.svg':similarityFlowImage,'images/v301-window-capture.svg':windowCaptureImage,'images/v310-sync-settings.png':syncSettingsImage,'images/v310-sync-actions.png':syncActionsImage,'images/v300-sync-flow.svg':syncFlowImage,'images/v316-guide-dark.png':guideDarkImage,'images/v316-guide-light.png':guideLightImage,'images/v105-revisions.svg':revisionsImage,'images/v104-math-ocr-input.png':mathOcrInputImage,'images/v104-math-ocr-workflow.svg':mathOcrWorkflowImage,'images/v103-ocr-index.svg':ocrIndexImage,'images/v102-clipper-settings.png':clipperSettingsImage,'images/v102-clipper-extension.png':clipperExtensionImage,'images/v101-scrolling.png':scrollingImage,'images/v021-projects.png':projectsImage,'images/v021-math.png':mathImage,'images/v100-capture.png':captureImage,'images/v100-library-grid.png':library_gridImage,'images/v100-library-list.png':library_listImage,'images/v100-editor.png':editorImage,'images/v100-card.png':cardImage,'images/v100-compare.png':compareImage,'images/v100-markdown.png':markdownImage,'images/v100-settings.png':settingsImage};
const guideAssets={};


function GuideBranch({nodes,depth,collapsed,active,searching,onToggle,onJump}:{nodes:GuideSection[];depth:number;collapsed:Set<number>;active:number|null;searching:boolean;onToggle:(offset:number)=>void;onJump:(offset:number)=>void}){
 return <ol className="guide-tree">{nodes.filter(n=>n.heading.depth<=depth).map(node=>{const h=node.heading,children=node.children.length>0&&h.depth<depth,expanded=searching||!collapsed.has(h.offset);return <li key={h.offset} data-guide-depth={h.depth}><div className="guide-tree-row">{children?<button className="guide-tree-toggle" aria-label={`${expanded?'收起':'展开'} ${h.text}`} aria-expanded={expanded} onClick={()=>onToggle(h.offset)}><ChevronRight size={13} style={{transform:expanded?'rotate(90deg)':undefined}}/></button>:<span className="guide-tree-spacer"/>}<button className="guide-tree-link" aria-current={active===h.offset?'location':undefined} onClick={()=>onJump(h.offset)}>{h.text}</button></div>{children&&expanded&&<GuideBranch nodes={node.children} depth={depth} collapsed={collapsed} active={active} searching={searching} onToggle={onToggle} onJump={onJump}/>}</li>;})}</ol>;
}
export function GuideView(){
 const targetChapter=useUiStore(s=>s.guideChapter),navigationSerial=useUiStore(s=>s.navigationSerial);
 const sections=useMemo(()=>guideSections(text),[]),body=useRef<HTMLDivElement>(null),[query,setQuery]=useState(''),[examplesOnly,setExamplesOnly]=useState(false),[active,setActive]=useState<number|null>(null),[figure,setFigure]=useState<{url:string;alt:string}|null>(null),[depth,setDepth]=useState(2),[collapsed,setCollapsed]=useState<Set<number>>(new Set());
 const openImage=useCallback((url:string,alt:string)=>setFigure({url,alt}),[setFigure]);
 const examples=useMemo(()=>guideExamples(sections),[sections]);
 const exampleCount=examples.flatMap(s=>s.children.flatMap(c=>c.children)).length;
 const visible=useMemo(()=>searchGuide(examplesOnly?examples:sections,query),[sections,examples,examplesOnly,query]);
 const jump=useCallback((offset:number)=>{setActive(offset);const node=body.current?.querySelector<HTMLElement>(`[data-heading-offset="${offset}"]`);if(node){node.tabIndex=-1;node.scrollIntoView({block:'start'});node.focus({preventScroll:true});}},[]);
 useEffect(()=>{const target=sections.find(s=>s.chapter===targetChapter);if(!target)return;const frame=requestAnimationFrame(()=>jump(target.heading.offset));return()=>cancelAnimationFrame(frame);},[targetChapter,navigationSerial,sections,jump]);
 useEffect(()=>{
  const host=body.current?.closest<HTMLElement>('.workspace-scroll'),offsets=new Set(guideOffsets(sections,4));let frame=0;
  const update=()=>{frame=0;if(!host)return;const top=host.getBoundingClientRect().top+80;const nodes=Array.from(body.current?.querySelectorAll<HTMLElement>('[data-heading-offset]')??[]).filter(n=>offsets.has(Number(n.dataset.headingOffset)));const current=nodes.filter(n=>n.getBoundingClientRect().top<=top).at(-1)??nodes[0];if(current)setActive(Number(current.dataset.headingOffset));};
  const scroll=()=>{if(!frame)frame=requestAnimationFrame(update);};host?.addEventListener('scroll',scroll,{passive:true});scroll();return()=>{host?.removeEventListener('scroll',scroll);cancelAnimationFrame(frame);};
 },[sections]);
 const shownDepth=query.trim()||examplesOnly?4:depth,shownActive=visibleGuideLocation(visible,active,shownDepth,query.trim()||examplesOnly?new Set():collapsed);
 const chapterIndex=sections.findIndex(s=>s.heading.offset===visibleGuideLocation(sections,active,2,new Set()));
 const toggle=(offset:number)=>setCollapsed(previous=>{const next=new Set(previous);if(next.has(offset))next.delete(offset);else next.add(offset);return next;});
 return <section className="guide-workspace" aria-label="完整使用指南"><header className="page-heading"><div><p className="guide-eyebrow"><BookOpen size={14}/> RESEARCHCLIP · 使用手册</p><h2>从截图到研究成果</h2><p>选一个任务，照着示例完成。每项功能都列出操作与完成标志。</p></div><span className="guide-edition">v{__APP_VERSION__} · Windows</span></header>
 <div className="guide-quick-start" aria-label="推荐阅读路线">{guideReadingRoutes.map((item,index)=><button key={item.section} onClick={()=>{setQuery('');setExamplesOnly(false);const section=sections.find(s=>s.chapter===item.section);if(section)jump(section.heading.offset);}}><span className="guide-route-number">{String(index+1).padStart(2,'0')}</span><div><strong>{item.title}</strong><span>{item.text}</span></div><ChevronRight size={16}/></button>)}</div>
 <div className="guide-layout"><nav aria-label="使用指南目录"><div className="guide-directory-heading"><strong>使用指南目录</strong><span>{sections.length} 章</span></div><div className="guide-directory-mode" role="group" aria-label="目录内容"><button aria-pressed={!examplesOnly} onClick={()=>setExamplesOnly(false)}>操作目录</button><button aria-pressed={examplesOnly} onClick={()=>setExamplesOnly(true)}>功能示例 <span>{exampleCount}</span></button></div><label className="guide-search-box"><Search size={15}/><input className="guide-search" aria-label="搜索指南章节" value={query} onChange={e=>setQuery(e.target.value)} placeholder={examplesOnly?'搜索示例，如“箭头”“DOI”…':'搜索标题或操作关键词…'}/></label><div className="guide-directory-tools"><label>目录层级<select aria-label="指南目录层级" disabled={examplesOnly} value={depth} onChange={e=>setDepth(Number(e.target.value))}><option value={2}>章节</option><option value={3}>章节与小节</option><option value={4}>详细操作步骤</option></select></label><div hidden={examplesOnly}><button onClick={()=>setCollapsed(new Set())}>展开全部</button><button onClick={()=>setCollapsed(new Set(sections.map(s=>s.heading.offset)))}>收起全部</button></div></div>
 {query&&<div className="guide-search-feedback" role="status"><span>{visible.length?`找到 ${visible.length} 个相关章节`:'没有匹配结果'}</span><button onClick={()=>setQuery('')}>清除</button></div>}
 {groups.map(group=>{const nodes=visible.filter(s=>group.sections.includes(s.chapter));return nodes.length?<div className="guide-topic-group" key={group.name}><h3>{group.name}</h3><GuideBranch nodes={nodes} depth={shownDepth} collapsed={collapsed} active={shownActive} searching={Boolean(query.trim())||examplesOnly} onToggle={toggle} onJump={jump}/></div>:null;})}{!visible.length&&<p className="guide-no-result">试试“版本”“插图”或“快捷键”。搜索会保留匹配小节的上级目录。</p>}</nav>
 <div ref={body} className="guide-content"><div className="guide-reading-bar"><span>{sections[chapterIndex]?.heading.text??"阅读使用指南"}</span><span className="guide-position">{chapterIndex+1} / {sections.length} 章</span><button className="secondary-button" onClick={()=>{body.current?.closest<HTMLElement>('.workspace-scroll')?.scrollTo({top:0});document.querySelector<HTMLInputElement>('.guide-search')?.focus({preventScroll:true});}}>回到目录</button></div><p className="guide-figure-hint">阅读提示：用“功能示例”快速演练，用“操作目录”查看完整步骤。示例需自行准备练习资料；图片按 Enter 放大，Esc 关闭。</p><MarkdownPreview markdown={text} assets={guideAssets} guideImages={guideImages} onImageOpen={openImage}/><nav className="guide-chapter-actions" aria-label="指南章节翻页"><button className="secondary-button" disabled={chapterIndex<=0} onClick={()=>jump(sections[chapterIndex-1].heading.offset)}>上一章</button><button className="secondary-button" disabled={chapterIndex>=sections.length-1} onClick={()=>jump(sections[chapterIndex+1].heading.offset)}>下一章</button></nav></div></div>
 {figure&&<Modal title="查看指南图片" onClose={()=>setFigure(null)}><div className="guide-image-full"><p>{figure.alt}</p><img src={figure.url} alt={figure.alt}/></div></Modal>}</section>;
}
