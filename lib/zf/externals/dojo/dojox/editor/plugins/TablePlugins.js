dojo.provide("dojox.editor.plugins.TablePlugins");
dojo.require("dijit._editor._Plugin");
dojo.require("dijit._editor.selection");
dojo.require("dijit.Menu");
dojo.require("dojo.i18n");
dojo.requireLocalization("dojox.editor.plugins", "TableDialog");

dojo.experimental("dojox.editor.plugins.TablePlugins");

//	summary:
//		A series of plugins that give the Editor the ability to create and edit 
//		HTML tables. See the end of this document for all avaiable plugins
// 		and dojox/editorPlugins/tests/editorTablePlugs.html for an example
//
//  example:
//		|	<div dojoType="dijit.Editor" plugins="[
//		|			'bold','italic','|',
//		|			{name: 'dojox.editor.plugins.TablePlugins', command: 'insertTable'},
//		|			{name: 'dojox.editor.plugins.TablePlugins', command: 'modifyTable'}
//		|		]">
//    	| 	   Editor text is here
//		|	</div>
//
//	TODO:	
//		Currently not supporting merging or splitting cells
//
//	FIXME: 	Undo is very buggy, and therefore unimeplented in all browsers 
//			except IE - which itself has only been lightly tested.
//
//  FIXME:	Selecting multiple table cells in Firefox looks to be impossible.
//			This affect the 'colorTableCell' plugin. Cells can still be 
//			colored individually or in rows.




dojo.declare("dojox.editor.plugins.GlobalTableHandler", dijit._editor._Plugin,{
	// summary:
	//		A global object that handles common tasks for all the plugins. Since 
	//		there are several plugins that are all calling common methods, it's preferable
	//		that they call a centralized location that either has a set variable or a 
	//		timeout to only repeat code-heavy calls when necessary.
	//
	tablesConnected:false,
	currentlyAvailable: false,
	alwaysAvailable:false,
	availableCurrentlySet:false,
	initialized:false,
	tableData: null,
	shiftKeyDown:false,
	editorDomNode: null,
	undoEnabled: dojo.isIE, //FIXME: 
	
	doMixins: function(){
		
		dojo.mixin(this.editor,{
			getAncestorElement: function(tagName){
				return dojo.withGlobal(this.window, "getAncestorElement",dijit._editor.selection, [tagName]);
			},
			hasAncestorElement: function(tagName){
				return dojo.withGlobal(this.window, "hasAncestorElement",dijit._editor.selection, [tagName]);
			},
			selectElement: function(elem){
				dojo.withGlobal(this.window, "selectElement",dijit._editor.selection, [elem]);
			},
			byId: function(id){
				return dojo.withGlobal(this.window, "byId", dojo, [id]);
			},
			query: function(arg, scope, returnFirstOnly){
				// this shortcut is dubious - not sure scoping is necessary
				var ar = dojo.withGlobal(this.window, "query", dojo, [arg, scope]);
				return (returnFirstOnly) ? ar[0] : ar;
			}
		});

	},
	initialize: function(editor){
		// summary:
		//		Initialize the global handler upon a plugin's first instance of setEditor
		//
		
		// 		All plugins will attempt initialization. We only need to do so once.
		if(this.initialized){ return; }
		
		this.initialized = true;
		this.editor = editor;
		
		
		this.editorDomNode = this.editor.iframe ? this.editor.document : this.editor.editNode;
		
		// RichText should have a mouseup connection to recognize drag-selections
		// Example would be selecting multiple table cells
		dojo.connect(this.editorDomNode , "mouseup", this.editor, "onClick"); 
		
		dojo.connect(this.editor, "onDisplayChanged", this, "checkAvailable");
		
		this.doMixins();
		this.connectDraggable();
	},
	
	getTableInfo: function(forceNewData){
		// summary
		//	Gets the table in focus
		//	Collects info on the table - see return params
		//
		if(forceNewData){ this._tempStoreTableData(false); }
		if(this.tableData){
			// tableData is set for a short amount of time, so that all 
			// plugins get the same return without doing the method over
			console.log("returning current tableData:", this.tableData);
			return this.tableData;	
		}
		var tr, trs, td, tds, tbl, cols, tdIndex, trIndex;

		td = this.editor.getAncestorElement("td");
		if(td){ tr = td.parentNode; }
		
		tbl = this.editor.getAncestorElement("table");
		//console.log("td:", td);console.log("tr:", tr);console.log("tbl:", tbl)
		
		tds = dojo.query("td", tbl);
		tds.forEach(function(d, i){
			if(td==d){tdIndex = i}
		});
		trs = dojo.query("tr", tbl);
		trs.forEach(function(r, i){
			if(tr==r){trIndex = i}
		});
		cols = tds.length/trs.length;
		var o = {
			tbl:tbl,		// focused table
			td:td,			// focused TD
			tr:tr,			// focused TR
			trs:trs,		// rows
			tds:tds,		// cells
			rows:trs.length,// row amount
			cols:cols,		// column amount
			tdIndex:tdIndex,// index of focused cell
			trIndex:trIndex,	// index of focused row
			colIndex:tdIndex%cols
		};
	console.log("NEW tableData:",o);
		this.tableData = o;
		this._tempStoreTableData(500);	
		return this.tableData;
	},
	
	connectDraggable: function(){
		// summary
		//	Detects drag-n-drop in the editor (could probably be moved to there)
		//	Currently only checks if item dragged was a TABLE, and removes its align attr
		//	DOES NOT WORK IN FF - it could - but FF's drag detection is a monster
		//
		if(!dojo.isIE){
			//console.warn("Drag and Drop is currently only detectable in IE.");
			return
		}
		
		// IE ONLY
		this.editorDomNode.ondragstart = dojo.hitch(this, "onDragStart");
		this.editorDomNode.ondragend = dojo.hitch(this, "onDragEnd");
		
		//NOTES:
		// FF _ Able to detect the drag-over object (the editor.domNode)
		//	Not able to detect an item's ondrag() event
		//	Don't know why - I actually got it working when there was an error
		//	Something to do with different documents or windows I'm sure
		//
		//console.log("connectDraggable", tbl);
		/*tbl.ondragstart=dojo.hitch(this, "onDragStart");
		
		tbl.addEventListener("dragstart", dojo.hitch(this, "onDragStart"), false);
		tbl.addEventListener("drag", dojo.hitch(this, "onDragStart2"), false);
		tbl.addEventListener("dragend", dojo.hitch(this, "onDragStart3"), false);
	
		dojo.withGlobal(this.editor.window, "selectElement",dijit._editor.selection, [tbl]);
		
		tbl.ondragstart = function(){
			console.log	("ondragstart");									
		};
		tbl.ondrag = function(){
			alert("drag")
			//console.log	("ondrag");									
		
		*/

		
	},
	onDragStart: function(){
		var e = window.event;
		if(!e.srcElement.id){
			e.srcElement.id = "tbl_"+(new Date().getTime());	
		}
		//console.log("onDragStart", e.srcElement.id);
	},
	onDragEnd: function(){
		// summary
		//	Detects that an object has been dragged into place
		// 	Currently, this code is only used for when a table is dragged
		//	and clears the "align" attribute, so that the table will look
		//	to be more in the place that the user expected.
		//	TODO: This code can be used for other things, most 
		//	notably UNDO, which currently is not quite usable.
		//	This code could also find itself in the Editor code when it is 
		//	complete.
		
		//console.log("onDragEnd");
		var e = window.event
		var node = e.srcElement;
		var id = node.id;
		var win = this.editor.window
		//console.log("NODE:", node.tagName, node.id,  dojo.attr(node, "align"));
		
		// clearing a table's align attr
		// TODO: when ondrag becomes more robust, this code block
		//	should move to its own method
		if(node.tagName.toLowerCase()=="table"){
			setTimeout(function(){
				var node =  dojo.withGlobal(win, "byId", dojo, [id]);
				dojo.removeAttr(node, "align");
				//console.log("set", node.tagName, dojo.attr(node, "align"))
			}, 100);
		}
	},
	checkAvailable: function(){
		// summary
		//	For table plugs
		//	Checking if a table or part of a table has focus so that 
		//	Plugs can change their status
		//
		if(this.availableCurrentlySet){
			// availableCurrentlySet is set for a short amount of time, so that all 
			// plugins get the same return without doing the method over
			//console.log("availableCurrentlySet:", this.availableCurrentlySet, "currentlyAvailable:", this.currentlyAvailable)
			return this.currentlyAvailable;
		}
		//console.log("G - checkAvailable...");
		
		if(!this.editor) {
			//console.log("editor not ready")
			return false;
		}
		if(this.alwaysAvailable) {
			//console.log(" return always available")
			return true;
		}
		
		this.currentlyAvailable = this.editor.hasAncestorElement("table");
		
		if(this.currentlyAvailable){
			this.connectTableKeys();
		}else{
			this.disconnectTableKeys();
		}
		
		this._tempAvailability(500);
		
		dojo.publish("available", [ this.currentlyAvailable ]);
		return this.currentlyAvailable;
	},
	
	_prepareTable: function(tbl){
		// 	For IE's sake, we are adding IDs to the TDs if none is there
		//	We go ahead and use it for other code for convenience
		//	
		var tds = this.editor.query("td", tbl);
		console.log("prep:", tds, tbl)
		if(!tds[0].id){
			tds.forEach(function(td, i){
				if(!td.id){
					td.id = "tdid"+i+this.getTimeStamp();
				}
			}, this);
		}
		return tds;
	},
	
	getTimeStamp: function(){
		return Math.floor(new Date().getTime() * 0.00000001);
	},
	
	_tempStoreTableData: function(type){
		// caching or clearing table data, depending on the arg
		//
		if(type===true){
			//store indefinitely	
		}else if(type===false){
			// clear object	
			this.tableData = null;
		}else if(type===undefined){
			console.warn("_tempStoreTableData must be passed an argument")	
		}else{ 
			// type is a number/ms
			setTimeout(dojo.hitch(this, function(){
				this.tableData = null;											 
			}), type);
		}
	},
	
	_tempAvailability: function(type){
			// caching or clearing availability, depending on the arg
		if(type===true){
			//store indefinitely
			this.availableCurrentlySet = true;
		}else if(type===false){
			// clear object	
			this.availableCurrentlySet = false;
		}else if(type===undefined){
			console.warn("_tempAvailability must be passed an argument")	
		}else{ 
			// type is a number/ms
			this.availableCurrentlySet = true;
			setTimeout(dojo.hitch(this, function(){
				this.availableCurrentlySet = false;											 
			}), type);
		}
		
	},
	
	connectTableKeys: function(){
		// summary
		//	When a table is in focus, start detecting keys
		//	Mainly checking for the TAB key so user can tab 
		//	through a table (blocking the browser's desire to
		//	tab away from teh editor completely)
		if(this.tablesConnected){ return; }
		this.tablesConnected = true;
		var node = (this.editor.iframe) ? this.editor.document : this.editor.editNode;
		this.cnKeyDn = dojo.connect(node, "onkeydown", this, "onKeyDown"); 
		this.cnKeyUp = dojo.connect(node, "onkeyup", this, "onKeyUp");
		 dojo.connect(node, "onkeypress", this, "onKeyUp");
		
	},
	disconnectTableKeys: function(){
		//console.log("disconnect")
		dojo.disconnect(this.cnKeyDn);
		dojo.disconnect(this.cnKeyUp);
		this.tablesConnected = false;
	},
	
	onKeyDown: function(evt){
		
		var key = evt.keyCode;
		//console.log(" -> DOWN:", key);
		if(key == 16) this.shiftKeyDown = true;
		if(key == 9) {console.log("TAB!:");
			var o = this.getTableInfo();
			//console.log("TAB ", o.tdIndex, o);
			// modifying the o.tdIndex in the tableData directly, because we may save it
			o.tdIndex = (this.shiftKeyDown) ? o.tdIndex-1 : tabTo = o.tdIndex+1;
			if(o.tdIndex>=0 && o.tdIndex<o.tds.length){
				
				this.editor.selectElement(o.tds[o.tdIndex]);
				
				// we know we are still within a table, so block the need
				//	to run the method
				this.currentlyAvailable = true;
				this._tempAvailability(true);
				// 
				this._tempStoreTableData(true);
				this.stopEvent = true;
			}else{
				//tabbed out of table
				this.stopEvent = false;
				this.onDisplayChanged();
			}
			if(this.stopEvent) {
				dojo.stopEvent(evt);
			}
		}
		
	},
	onKeyUp: function(evt){
		var key = evt.keyCode;
		//console.log(" -> UP:", key)
		if(key == 16) this.shiftKeyDown = false;
		if(key == 37 || key == 38 || key == 39 || key == 40 ){
			// user can arrow or tab out of table - need to recheck
			this.onDisplayChanged();
		}
		if(key == 9 && this.stopEvent) dojo.stopEvent(evt);
	},
	onDisplayChanged: function(){
		//console.log("onDisplayChanged")
		this.currentlyAvailable = false;
		this._tempStoreTableData(false);
		this._tempAvailability(false);
		this.checkAvailable();
	}
});
// global:
tablePluginHandler = new dojox.editor.plugins.GlobalTableHandler(); //FIXME: no globals.

dojo.declare("dojox.editor.plugins.TablePlugins",
	dijit._editor._Plugin,
	{
		//summary: 
		// A collection of Plugins for inserting and modifying tables in the Editor
		// See end of this document for all avaiable plugs
		// and dojox/editorPlugins/tests/editorTablePlugs.html for an example
		//
		//	NOT IMPLEMENTED: Not handling cell merge, span or split
		//
		
		iconClassPrefix: "editorIcon",
		useDefaultCommand: false,
		buttonClass: dijit.form.Button,
		commandName:"",
		label:"",
		alwaysAvailable:false,
		undoEnabled:false,
		
		constructor: function(){
			// summary 
			//		Initialize certain plugins
			//
			switch(this.commandName){
				
				case "colorTableCell":
					this.buttonClass = dijit.form.DropDownButton;
					this.dropDown = new dijit.ColorPalette();
					this.connect(this.dropDown, "onChange", function(color){
						this.modTable(null, color);
					});
					break;
				
				case "modifyTable":
					this.buttonClass = dijit.form.DropDownButton;
					this.modTable = this.launchModifyDialog;
					break;
				
				case "insertTable":
					this.alwaysAvailable = true;
					this.buttonClass = dijit.form.DropDownButton;
					this.modTable = this.launchInsertDialog;
					break;
				
				case "tableContextMenu":
					this.connect(this, "setEditor", function(){
						this._createContextMenu();
						this.button.domNode.style.display = "none";
					});
					break;
			}
			
			dojo.subscribe("available", this, "onDisplayChanged");
			
		},
		onDisplayChanged: function(withinTable){
			// subscribed to from the global object's publish method
			//
			//console.log("onDisplayChanged", this.commandName);
			if(!this.alwaysAvailable){
				this.available = withinTable;
				this.button.attr('disabled', !this.available);
			}
		},
		
		setEditor: function(){
			this.inherited(arguments);
			this.onEditorLoaded();
		},
		onEditorLoaded: function(){
			//stub
			// call global object to initialize it
			tablePluginHandler.initialize(this.editor);
		},
		
		_createContextMenu: function(){
			// summary
			//		Building context menu for right-click shortcuts within a table
			//
			var node = dojo.isFF ? this.editor.editNode : this.editor.document.firstChild;
			
			pMenu = new dijit.Menu({targetNodeIds:[node], id:"progMenu", contextMenuForWindow:dojo.isIE});
			var _M = dijit.MenuItem;
			var messages = dojo.i18n.getLocalization("dojox.editor.plugins", "TableDialog", this.lang);
			pMenu.addChild(new _M({label: messages.selectTableLabel, onClick: dojo.hitch(this, "selectTable")}));
			pMenu.addChild(new dijit.MenuSeparator());
			
			pMenu.addChild(new _M({label: messages.insertTableRowBeforeLabel, onClick: dojo.hitch(this, "modTable", "insertTableRowBefore" )}));
			pMenu.addChild(new _M({label: messages.insertTableRowAfterLabel, onClick: dojo.hitch(this, "modTable", "insertTableRowAfter" )}));
			pMenu.addChild(new _M({label: messages.insertTableColumnBeforeLabel, onClick: dojo.hitch(this, "modTable", "insertTableColumnBefore" )}));
			pMenu.addChild(new _M({label: messages.insertTableColumnAfterLabel, onClick: dojo.hitch(this, "modTable", "insertTableColumnAfter" )}));
			pMenu.addChild(new dijit.MenuSeparator());
			pMenu.addChild(new _M({label: messages.deleteTableRowLabel, onClick: dojo.hitch(this, "modTable", "deleteTableRow" )}));
			pMenu.addChild(new _M({label: messages.deleteTableColumnLabel, onClick: dojo.hitch(this, "modTable", "deleteTableColumn" )}));
			
			// overwriting this method, as the menu's coordinates
			// are not accurate in the editor's iframe
			// FIXME: Works well in IE - all others, sometimes inaccurate.
			pMenu._openMyself = function(e){ 
					
				if(!tablePluginHandler.checkAvailable()){ return; }

				if(this.leftClickToOpen && e.button>0){
					return;
				}
				dojo.stopEvent(e);
				
				var x,y;
				if(dojo.isIE){
					x = e.x;
					y = e.y;
				}else{
					x = e.screenX;
					y = e.screenY + 25;
				}
				var self=this;
				var savedFocus = dijit.getFocus(this);
				function closeAndRestoreFocus(){
					// user has clicked on a menu or popup
					dijit.focus(savedFocus);
					dijit.popup.close(self);
				}
				var res = dijit.popup.open({
					popup: this,
					x: x,
					y: y,
					onExecute: closeAndRestoreFocus,
					onCancel: closeAndRestoreFocus,
					orient: this.isLeftToRight() ? 'L' : 'R'
				});
				var v = dijit.getViewport();
				if(res.y+res.h>v.h){
					if(e.screenY-res.h>=0){
						y = e.screenY - res.h;	
					}else{
						y = 0;	
					}
					dijit.popup.close(this);
					// FIXME: Not very DRY here.
					//	Reopening popup if the location was not good.
					res = dijit.popup.open({
						popup: this,
						x: x,
						y: y,
						onExecute: closeAndRestoreFocus,
						onCancel: closeAndRestoreFocus,
						orient: this.isLeftToRight() ? 'L' : 'R'
					});
				}
				console.log(dijit.getViewport());
				this.focus();
		
				this._onBlur = function(){
					this.inherited('_onBlur', arguments);
					dijit.popup.close(this);
				}
			}
			this.menu = pMenu;
		},
		
		
		selectTable: function(){
			// selects table that is in focus 
			var o = this.getTableInfo();
			dojo.withGlobal(this.editor.window, "selectElement",dijit._editor.selection, [o.tbl]);
		},
		launchInsertDialog: function(){
			var w = new dojox.editor.plugins.EditorTableDialog({});
			w.show();
			var c = dojo.connect(w, "onBuildTable", this, function(obj){
				dojo.disconnect(c);
				
				var res = this.editor.execCommand('inserthtml', obj.htmlText);
				
				// commenting this line, due to msg below
				//var td = this.editor.query("td", this.editor.byId(obj.id));
				
				//HMMMM.... This throws a security error now. didn't used to.
				//this.editor.selectElement(td);
			});
		},
		
		launchModifyDialog: function(){
			var o = this.getTableInfo();
			console.log("LAUNCH DIALOG")
			var w = new dojox.editor.plugins.EditorModifyTableDialog({table:o.tbl});
			w.show();
			this.connect(w, "onSetTable", function(color){
				// uhm... not sure whats going on here...
				var o = this.getTableInfo();
				console.log("set color:", color)
				dojo.attr(o.td, "bgcolor", color);
			});
		},
		
		_initButton: function(){
			this.command = this.commandName;
			
			this.label = this.editor.commands[this.command] = this._makeTitle(this.command);
			this.inherited(arguments);
			delete this.command;
			
			if(this.commandName != "colorTableCell") this.connect(this.button.domNode, "click", "modTable");
			if(this.commandName=="tableContextMenu") this.button.domNode.display = "none";
			
			this.onDisplayChanged(false);
		},
		
		modTable: function(cmd, args){
			// summary
			//	Where each plugin performs its action
			//	Note: not using execCommand. In spite of their presence in the 
			//	Editor as query-able plugins, I was not able to find any evidence
			//	that they are supported (especially in NOT IE). If they are 
			//	supported in other browsers, it may help with the undo problem.
			//
			this.begEdit();
			var o = this.getTableInfo();
			var sw = (dojo.isString(cmd))?cmd : this.commandName;
			var r, c, i;
			var adjustColWidth = false;
			//console.log("modTable:", sw)
			
			switch(sw){
				case "insertTableRowBefore":
					r = o.tbl.insertRow(o.trIndex);
					for(i=0;i<o.cols;i++){
						c = r.insertCell(-1);
						c.innerHTML = "&nbsp;";
					}
					break;
				case "insertTableRowAfter":
					r = o.tbl.insertRow(o.trIndex+1);
					for(i=0;i<o.cols;i++){
						c = r.insertCell(-1);
						c.innerHTML = "&nbsp;";
					}
					break;
				case "insertTableColumnBefore":
					o.trs.forEach(function(r){
						c = r.insertCell(o.colIndex);
						c.innerHTML = "&nbsp;";
					});
					adjustColWidth = true;
					break;
				case "insertTableColumnAfter":
					o.trs.forEach(function(r){
						c = r.insertCell(o.colIndex+1);
						c.innerHTML = "&nbsp;";
					});
					adjustColWidth = true;
					break;
				case "deleteTableRow":
					o.tbl.deleteRow(o.trIndex);
					console.log("TableInfo:", this.getTableInfo())
					break;
				case "deleteTableColumn":
					o.trs.forEach(function(tr){
						tr.deleteCell(o.colIndex);
					});
					adjustColWidth = true;
					break;
				case "colorTableCell":
					// The one plugin that really needs use of the very verbose
					//	getSelectedCells()
					var tds = this.getSelectedCells(o.tbl);
					dojo.forEach(tds, function(td){
						dojo.style(td, "backgroundColor", args);				   
					});
					
					break;
				case "modifyTable":
					break;
				case "insertTable":
					break
				
			}
			if(adjustColWidth){
				this.makeColumnsEven();
			}
			this.endEdit();
		},
		
		// UNDO HANDLERS
		// Only works in IE
		begEdit: function(){
			if(tablePluginHandler.undoEnabled){
				console.log("UNDO:", this.editor.customUndo)
				if(this.editor.customUndo){
					this.editor.beginEditing();
				}else{
					this.valBeforeUndo = this.editor.getValue();
					console.log("VAL:", this.valBeforeUndo)
					
				}
			}
		},
		endEdit: function(){
			if(tablePluginHandler.undoEnabled){
				if(this.editor.customUndo){
					this.editor.endEditing();
				}else{
					// This code ALMOST works for undo - 
					//	It seems to only work for one step
					//	back in history however
					var afterUndo = this.editor.getValue();
					//this.editor.execCommand("inserthtml", "<p>mike</p>");
					this.editor.setValue(this.valBeforeUndo);
					this.editor.replaceValue(afterUndo);
				}
				
				this.editor.onDisplayChanged();
			}
		},
		
		makeColumnsEven: function(){
			//summary 
			//	After changing column amount, change widths to
			//	keep columns even
			//
			// the timeout helps prevent an occasional snafu
			setTimeout(dojo.hitch(this, function(){
				var o = this.getTableInfo(true);
				var w = Math.floor(100/o.cols);
				o.tds.forEach(function(d){
					dojo.attr(d, "width", w+"%");
				});
			}), 10);
		},
		
		getTableInfo: function(forceNewData){
			// summary
			//	Gets the table in focus
			//	Collects info on the table - see return params
			//
			return tablePluginHandler.getTableInfo(forceNewData);
		},
		_makeTitle: function(str){
			// Parses the commandName into a Title
			//	based on camelCase
			var s = str.split(""), ns = [];
			dojo.forEach(str, function(c, i){
				if(c.charCodeAt(0)<91 && i>0 && ns[i-1].charCodeAt(0)!=32){
					ns.push(" ");
				}
				if(i==0) c = c.toUpperCase();
				ns.push(c);
			});
			return ns.join("");	
		},
		
		
		
		getSelectedCells: function(){
			// summary
			//	Gets the selected cells from the passed table
			//	Returns: array of TDs or empty array
			//
			
			var cells = [];
			var tbl = this.getTableInfo().tbl;
			var tds = tablePluginHandler._prepareTable(tbl);
			var e = this.editor;
			var r;
			
			if(!dojo.isIE){
				
				r = dijit.range.getSelection(e.window);
				
				var foundFirst=false;
				var foundLast=false;
				
				if(r.anchorNode && r.anchorNode.tagName && r.anchorNode.tagName.toLowerCase()=="tr"){
					// Firefox
					// 	Geez - and I thought IE was hard....
					// 	In the Editor, FF refuses to grab a multiple cell selection at the TD level
					//	in spite of multiple selection techniques
					// 	Resorting to going ahead with its TR selection and selecting
					// 	the entire row
					//	Has no problem with individual cell selection though
					//
					var trs = dojo.query("tr", tbl);
					var rows = [];
					trs.forEach(function(tr, i){
						
						if(!foundFirst && (tr == r.anchorNode || tr == r.focusNode)){
							rows.push(tr);
							foundFirst = true;
							if(r.anchorNode == r.focusNode){
								foundLast = true;
							}
						}else if(foundFirst && !foundLast){
							rows.push(tr);
							if(tr == r.anchorNode || tr == r.focusNode){
								foundLast = true;
							}
						}
					});
					dojo.forEach(rows, function(tr){
						cells = cells.concat(dojo.query("td", tr));					
					}, this);
				}else{
					// Safari
					//	Yay! It works like expected
					//	Getting the cells from anchorNode to focusNode
					//
					tds.forEach(function(td, i){
						if(!foundFirst && (td.id == r.anchorNode.parentNode.id || td.id == r.focusNode.parentNode.id)){
							cells.push(td);
							foundFirst = true;
							if(r.anchorNode.parentNode.id == r.focusNode.parentNode.id){
								foundLast = true;
							}
						}else if(foundFirst && !foundLast){
							cells.push(td)
							if(td.id == r.focusNode.parentNode.id || td.id == r.anchorNode.parentNode.id){
								foundLast = true;
							}
						}
					});
					console.log("SAF CELLS:", cells);
				}
				
			}
			
			if(dojo.isIE){
				// IE
				// 	Although the code is tight - there's some funkiness here
				//	Can only get the htmlText, so we add IDs to the cells (above)
				//	And pull them from the htmlText, then search for those cells
				//
				r = document.selection.createRange()
				
				var str = r.htmlText.match(/id=\w*/g);
				dojo.forEach(str, function(a){
					var id = a.substring(3, a.length);
					cells.push(e.byId(id));
				}, this)
				
			}
			return cells;
		}
		
	}
);

dojo.provide("dojox.editor.plugins.EditorTableDialog");
dojo.require("dijit.Dialog");
dojo.require("dijit.form.TextBox");
dojo.require("dijit.form.FilteringSelect");
dojo.require("dijit.form.Button");

dojo.declare("dojox.editor.plugins.EditorTableDialog", [dijit.Dialog], {
	// summary
	//	Dialog box with options for table creation
	//
	baseClass:"EditorTableDialog",
				
	widgetsInTemplate:true,
	templatePath: dojo.moduleUrl("dojox.editor.plugins", "resources/insertTable.html"),

	postMixInProperties: function(){
		var messages = dojo.i18n.getLocalization("dojox.editor.plugins", "TableDialog", this.lang);
		dojo.mixin(this, messages);
		this.inherited(arguments);
	},

	postCreate: function(){
		dojo.addClass(this.domNode, this.baseClass); //FIXME - why isn't Dialog accepting the baseClass?
		this.inherited(arguments);
	},

	onInsert: function(){
		console.log("insert");
		
		var rows = 		this.selectRow.attr("value") || 1,
			cols = 		this.selectCol.attr("value") || 1,
			width = 	this.selectWidth.attr("value"),
			widthType = this.selectWidthType.attr("value"),
			border = 	this.selectBorder.attr("value"),
			pad = 		this.selectPad.attr("value"),
			space = 	this.selectSpace.attr("value"),
			_id =		"tbl_"+(new Date().getTime()),
			t = '<table id="'+_id+'"width="'+width+((widthType=="percent")?'%':'')+'" border="'+border+'" cellspacing="'+space+'" cellpadding="'+pad+'">\n';
		
		for(var r=0;r<rows;r++){
			t += '\t<tr>\n';
			for(var c=0;c<cols;c++){
				t += '\t\t<td width="'+(Math.floor(100/cols))+'%">&nbsp;</td>\n';
			}
			t += '\t</tr>\n';
		}
		t += '</table>';
		
		//console.log(t);
		this.onBuildTable({htmlText:t, id:_id});
		this.hide();
	},
	onBuildTable: function(tableText){
		//stub
	}
});


dojo.provide("dojox.editor.plugins.EditorModifyTableDialog");
dojo.require("dijit.ColorPalette");

dojo.declare("dojox.editor.plugins.EditorModifyTableDialog", [dijit.Dialog], {
	
	// summary
	//	Dialog box with options for editing a table
	//
	
	baseClass:"EditorTableDialog",

	widgetsInTemplate:true,
	table:null, //html table to be modified
	tableAtts:{},
	templatePath: dojo.moduleUrl("dojox.editor.plugins", "resources/modifyTable.html"),

	postMixInProperties: function(){
		var messages = dojo.i18n.getLocalization("dojox.editor.plugins", "TableDialog", this.lang);
		dojo.mixin(this, messages);
		this.inherited(arguments);
	},

	postCreate: function(){
		dojo.addClass(this.domNode, this.baseClass); //FIXME - why isn't Dialog accepting the baseClass?
		this.inherited(arguments);
		
		this.connect(this.borderCol, "click", function(){
			var div = document.createElement("div");
			var w = new dijit.ColorPalette({}, div);
			dijit.popup.open({popup:w, around:this.borderCol});
			this.connect(w, "onChange", function(color){
				dijit.popup.close(w);
				this.setBrdColor(color);
			});
		});
		this.connect(this.backgroundCol, "click", function(){
			var div = document.createElement("div");
			var w = new dijit.ColorPalette({}, div);
			dijit.popup.open({popup:w, around:this.backgroundCol});
			this.connect(w, "onChange", function(color){
				dijit.popup.close(w);
				this.setBkColor(color);
			});
		});
		
		this.setBrdColor(dojo.attr(this.table, "bordercolor"));
		this.setBkColor(dojo.attr(this.table, "bgcolor"));
		var w = dojo.attr(this.table, "width");
		var p = "pixels";
		if(w.indexOf("%")>-1){
			p = "percent"
			w = w.replace(/%/, "");
		}
		
		this.selectWidth.attr("value", w);
		this.selectWidthType.attr("value", p);
		
		this.selectBorder.attr("value", dojo.attr(this.table, "border"));
		this.selectPad.attr("value", dojo.attr(this.table, "cellpadding"));
		this.selectSpace.attr("value", dojo.attr(this.table, "cellspacing"));
		this.selectAlign.attr("value", dojo.attr(this.table, "align"));
	},
	
	setBrdColor: function(color){
		this.brdColor = color;
		dojo.style(this.borderCol, "backgroundColor", color);
	},
	
	setBkColor: function(color){
		this.bkColor = color;
		dojo.style(this.backgroundCol, "backgroundColor", color);
	},
	onSet: function(){
		dojo.attr(this.table, "bordercolor", this.brdColor);
		dojo.attr(this.table, "bgcolor", this.bkColor);
		dojo.attr(this.table, "width", (this.selectWidth.attr("value") + ((this.selectWidthType.attr("value")=="pixels")?"":"%") ));
		dojo.attr(this.table, "border", this.selectBorder.attr("value"));
		dojo.attr(this.table, "cellpadding", this.selectPad.attr("value"));
		dojo.attr(this.table, "cellspacing", this.selectSpace.attr("value"));
		dojo.attr(this.table, "align", this.selectAlign.attr("value"));
		
		this.hide();
	},
	onSetTable: function(tableText){
		//stub
	}
});




dojo.subscribe(dijit._scopeName + ".Editor.getPlugin",null,function(o){
	if(o.plugin){ return; }
	// make first charcter lower case
	if(o.args && o.args.command){
		var cmd = o.args.command.charAt(0).toLowerCase()+o.args.command.substring(1,o.args.command.length);
		
		switch(cmd){
			case "insertTableRowBefore":
			case "insertTableRowAfter":
			case "insertTableColumnBefore":
			case "insertTableColumnAfter":
			case "deleteTableRow":
			case "deleteTableColumn":
			case "colorTableCell":
			case "modifyTable":
			case "insertTable":
			case "tableContextMenu":
			
				o.plugin = new dojox.editor.plugins.TablePlugins({commandName: cmd});
				break;
		}
	}
});
