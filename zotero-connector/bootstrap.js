/* global Zotero, Services */
var connector;
var connectorScope;
function install(){}
async function startup({id,version,rootURI}){
 // Zotero invokes lifecycle hooks with its per-addon sandbox as this.
 // Load into that real global so connector.js exports never depend on a plain-object scope chain.
 connectorScope=globalThis;
 Services.scriptloader.loadSubScript(rootURI+'connector.js',connectorScope);
 connector=new connectorScope.ResearchClipConnector({Zotero,Services,id,version,rootURI});
 await connector.start();
}
function onMainWindowLoad({window}){connector?.addWindow(window);}
function onMainWindowUnload({window}){connector?.removeWindow(window);}
async function shutdown(){await connector?.shutdown();connector=null;connectorScope=null;}
function uninstall(){}
