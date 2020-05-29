dojo.provide("dijit.Dialog");

dojo.require("dojo.dnd.move");
dojo.require("dojo.dnd.TimedMoveable");
dojo.require("dojo.fx");

dojo.require("dijit._Widget");
dojo.require("dijit._Templated");
dojo.require("dijit.layout.ContentPane");
dojo.require("dijit.form.Form");
dojo.requireLocalization("dijit", "common");

dojo.declare(
	"dijit.DialogUnderlay",
	[dijit._Widget, dijit._Templated],
	{
		// summary: The component that grays out the screen behind the dialog
	
		// Template has two divs; outer div is used for fade-in/fade-out, and also to hold background iframe.
		// Inner div has opacity specified in CSS file.
		templateString: "<div class='dijitDialogUnderlayWrapper' id='${id}_wrapper'><div class='dijitDialogUnderlay ${class}' id='${id}' dojoAttachPoint='node'></div></div>",

		attributeMap: {},

		postCreate: function(){
			// summary: Append the underlay to the body
			dojo.body().appendChild(this.domNode);
			this.bgIframe = new dijit.BackgroundIframe(this.domNode);
		},

		layout: function(){
			// summary: Sets the background to the size of the viewport
			//
			// description:
			//	Sets the background to the size of the viewport (rather than the size
			//	of the document) since we need to cover the whole browser window, even
			//	if the document is only a few lines long.

			var viewport = dijit.getViewport();
			var is = this.node.style,
				os = this.domNode.style;

			os.top = viewport.t + "px";
			os.left = viewport.l + "px";
			is.width = viewport.w + "px";
			is.height = viewport.h + "px";

			// process twice since the scroll bar may have been removed
			// by the previous resizing
			var viewport2 = dijit.getViewport();
			if(viewport.w != viewport2.w){ is.width = viewport2.w + "px"; }
			if(viewport.h != viewport2.h){ is.height = viewport2.h + "px"; }
		},

		show: function(){
			// summary: Show the dialog underlay
			this.domNode.style.display = "block";
			this.layout();
			if(this.bgIframe.iframe){
				this.bgIframe.iframe.style.display = "block";
			}
		},

		hide: function(){
			// summary: hides the dialog underlay
			this.domNode.style.display = "none";
			if(this.bgIframe.iframe){
				this.bgIframe.iframe.style.display = "none";
			}
		},

		uninitialize: function(){
			if(this.bgIframe){
				this.bgIframe.destroy();
			}
		}
	}
);


dojo.declare("dijit._DialogMixin", null,
	{
		attributeMap: dijit._Widget.prototype.attributeMap,

		// execute: Function
		//	User defined function to do stuff when the user hits the submit button
		execute: function(/*Object*/ formContents){},

		// onCancel: Function
		//      Callback when user has canceled dialog, to notify container
		//      (user shouldn't override)
		onCancel: function(){},

		// onExecute: Function
		//	Callback when user is about to execute dialog, to notify container
		//	(user shouldn't override)
		onExecute: function(){},

		_onSubmit: function(){
			// summary: callback when user hits submit button
			this.onExecute();	// notify container that we are about to execute
			this.execute(this.attr('value'));
		},

		_getFocusItems: function(/*Node*/ dialogNode){
			// find focusable Items each time a dialog is opened
			
			var elems = dijit._getTabNavigable(dojo.byId(dialogNode));
			this._firstFocusItem = elems.lowest || elems.first || dialogNode;
			this._lastFocusItem = elems.last || elems.highest || this._firstFocusItem;
			if(dojo.isMoz && this._firstFocusItem.tagName.toLowerCase() == "input" && dojo.attr(this._firstFocusItem, "type").toLowerCase() == "file"){
					//FF doesn't behave well when first element is input type=file, set first focusable to dialog container
					dojo.attr(dialogNode, "tabindex", "0");
					this._firstFocusItem = dialogNode;
			}
		}
	}
);

dojo.declare(
	"dijit.Dialog",
	[dijit.layout.ContentPane, dijit._Templated, dijit.form._FormMixin, dijit._DialogMixin],
	{
		// summary: A modal dialog Widget
		//
		// description:
		//	Pops up a modal dialog window, blocking access to the screen
		//	and also graying out the screen Dialog is extended from
		//	ContentPane so it supports all the same parameters (href, etc.)
		//
		// example:
		// |	<div dojoType="dijit.Dialog" href="test.html"></div>
		//
		// example:
		// |	var foo = new dijit.Dialog({ title: "test dialog", content: "test content" };
		// |	dojo.body().appendChild(foo.domNode);
		// |	foo.startup();
		
		templateString: null,
		templatePath: dojo.moduleUrl("dijit", "templates/Dialog.html"),
		attributeMap: dojo.mixin(dojo.clone(dijit._Widget.prototype.attributeMap), {
			title: [{node: "titleNode", type: "innerHTML"}, {node: "titleBar", type: "attribute"}]
		}),

		// open: Boolean
		//		is True or False depending on state of dialog
		open: false,

		// duration: Integer
		//		The time in milliseconds it takes the dialog to fade in and out
		duration: dijit.defaultDuration,

		// refocus: Boolean
		// 		A Toggle to modify the default focus behavior of a Dialog, which
		// 		is to re-focus the element which had focus before being opened.
		//		False will disable refocusing. Default: true
		refocus: true,
		
		// autofocus: Boolean
		// 		A Toggle to modify the default focus behavior of a Dialog, which
		// 		is to focus on the first dialog element after opening the dialog.
		//		False will disable autofocusing. Default: true
		autofocus: true,

		// _firstFocusItem: DomNode
		//		The pointer to the first focusable node in the dialog
		_firstFocusItem:null,
		
		// _lastFocusItem: DomNode
		//		The pointer to which node has focus prior to our dialog
		_lastFocusItem:null,

		// doLayout: Boolean
		//		Don't change this parameter from the default value.
		//		This ContentPane parameter doesn't make sense for Dialog, since Dialog
		//		is never a child of a layout container, nor can you specify the size of
		//		Dialog in order to control the size of an inner widget. 
		doLayout: false,

		// draggable: Boolean
		//		Toggles the moveable aspect of the Dialog. If true, Dialog
		//		can be moved by it's title. If false it will remain centered
		//		in the viewport.
		draggable: true,

		postMixInProperties: function(){
			var _nlsResources = dojo.i18n.getLocalization("dijit", "common");
			dojo.mixin(this, _nlsResources);
			this.inherited(arguments);
		},

		postCreate: function(){
			var s = this.domNode.style;
			s.visibility = "hidden";
			s.position = "absolute";
			s.display = "";
			s.top = "-9999px";
			dojo.body().appendChild(this.domNode);

			this.inherited(arguments);

			this.connect(this, "onExecute", "hide");
			this.connect(this, "onCancel", "hide");
			this._modalconnects = [];
		},

		onLoad: function(){
			// summary: when href is specified we need to reposition the dialog after the data is loaded
			this._position();
			this.inherited(arguments);
		},

		_endDrag: function(e){
			// summary: Called after dragging the Dialog. Calculates the relative offset
			//		of the Dialog in relation to the viewport. 
			if(e && e.node && e.node === this.domNode){
				var vp = dijit.getViewport(); 
				var p = e._leftTop || dojo.coords(e.node,true);
				this._relativePosition = {
					t: p.t - vp.t,
					l: p.l - vp.l
				}			
			}
		},
		
		_setup: function(){
			// summary: 
			//		stuff we need to do before showing the Dialog for the first
			//		time (but we defer it until right beforehand, for
			//		performance reasons)

			var node = this.domNode;

			if(this.titleBar && this.draggable){
				this._moveable = (dojo.isIE == 6) ?
					new dojo.dnd.TimedMoveable(node, { handle: this.titleBar }) :	// prevent overload, see #5285
					new dojo.dnd.Moveable(node, { handle: this.titleBar, timeout: 0 });
				dojo.subscribe("/dnd/move/stop",this,"_endDrag");
			}else{
				dojo.addClass(node,"dijitDialogFixed"); 
			}

			this._underlay = new dijit.DialogUnderlay({
				id: this.id+"_underlay",
				"class": dojo.map(this["class"].split(/\s/), function(s){ return s+"_underlay"; }).join(" ")
			});

			var underlay = this._underlay;

			this._fadeIn = dojo.fadeIn({
				node: node,
				duration: this.duration,
				onBegin: dojo.hitch(underlay, "show")
			 });

			this._fadeOut = dojo.fadeOut({
				node: node,
				duration: this.duration,
				onEnd: function(){
					node.style.visibility="hidden";
					node.style.top = "-9999px";
					underlay.hide();
				}
			 });
		},

		uninitialize: function(){
			if(this._fadeIn && this._fadeIn.status() == "playing"){
				this._fadeIn.stop();
			}
			if(this._fadeOut && this._fadeOut.status() == "playing"){
				this._fadeOut.stop();
			}
			if(this._underlay){
				this._underlay.destroy();
			}
			if(this._moveable){
				this._moveable.destroy();
			}
		},

		_size: function(){
			// summary:
			// 		Make sure the dialog is small enough to fit in viewport.

			var mb = dojo.marginBox(this.domNode);
			var viewport = dijit.getViewport();
			if(mb.w >= viewport.w || mb.h >= viewport.h){
				dojo.style(this.containerNode, {
					width: Math.min(mb.w, Math.floor(viewport.w * 0.75))+"px",
					height: Math.min(mb.h, Math.floor(viewport.h * 0.75))+"px",
					overflow: "auto",
					position: "relative"	// workaround IE bug moving scrollbar or dragging dialog
				});
			}
		},

		_position: function(){
			// summary: Position modal dialog in the viewport. If no relative offset
			//		in the viewport has been determined (by dragging, for instance),
			//		center the node. Otherwise, use the Dialog's stored relative offset,
			//		and position the node to top: left: values based on the viewport.
			if(!dojo.hasClass(dojo.body(),"dojoMove")){
				
				var node = this.domNode;
				var viewport = dijit.getViewport();
				var p = this._relativePosition;
				var mb = p ? null : dojo.marginBox(node);
				dojo.style(node,{
					left: Math.floor(viewport.l + (p ? p.l : (viewport.w - mb.w) / 2)) + "px",
					top: Math.floor(viewport.t + (p ? p.t : (viewport.h - mb.h) / 2)) + "px"
				});
			}

		},

		_onKey: function(/*Event*/ evt){
			// summary: handles the keyboard events for accessibility reasons
			if(evt.charOrCode){
				var dk = dojo.keys;
				var node = evt.target;
				if (evt.charOrCode === dk.TAB){
					this._getFocusItems(this.domNode);
				}
				var singleFocusItem = (this._firstFocusItem == this._lastFocusItem);
				// see if we are shift-tabbing from first focusable item on dialog
				if(node == this._firstFocusItem && evt.shiftKey && evt.charOrCode === dk.TAB){
					if(!singleFocusItem){
						dijit.focus(this._lastFocusItem); // send focus to last item in dialog
					}
					dojo.stopEvent(evt);
				}else if(node == this._lastFocusItem && evt.charOrCode === dk.TAB && !evt.shiftKey){
					if (!singleFocusItem){
						dijit.focus(this._firstFocusItem); // send focus to first item in dialog
					}
					dojo.stopEvent(evt);
				}else{
					// see if the key is for the dialog
					while(node){
						if(node == this.domNode){
							if(evt.charOrCode == dk.ESCAPE){
								this.onCancel(); 
							}else{
								return; // just let it go
							}
						}
						node = node.parentNode;
					}
					// this key is for the disabled document window
					if(evt.charOrCode !== dk.TAB){ // allow tabbing into the dialog for a11y
						dojo.stopEvent(evt);
					// opera won't tab to a div
					}else if(!dojo.isOpera){
						try{
							this._firstFocusItem.focus();
						}catch(e){ /*squelch*/ }
					}
				}
			}
		},

		show: function(){
			// summary: display the dialog

			if(this.open){ return; }
			
			// first time we show the dialog, there's some initialization stuff to do			
			if(!this._alreadyInitialized){
				this._setup();
				this._alreadyInitialized=true;
			}

			if(this._fadeOut.status() == "playing"){
				this._fadeOut.stop();
			}

			this._modalconnects.push(dojo.connect(window, "onscroll", this, "layout"));
			this._modalconnects.push(dojo.connect(window, "onresize", this, "layout"));
			this._modalconnects.push(dojo.connect(dojo.doc.documentElement, "onkeypress", this, "_onKey"));

			dojo.style(this.domNode, {
				opacity:0,
				visibility:""
			});
			
			this.open = true;
			this._loadCheck(); // lazy load trigger

			this._size();
			this._position();

			this._fadeIn.play();

			this._savedFocus = dijit.getFocus(this);

			if(this.autofocus){
				// find focusable Items each time dialog is shown since if dialog contains a widget the 
				// first focusable items can change
				this._getFocusItems(this.domNode);
	
				// set timeout to allow the browser to render dialog
				setTimeout(dojo.hitch(dijit,"focus",this._firstFocusItem), 50);
			}
		},

		hide: function(){
			// summary: Hide the dialog

			// if we haven't been initialized yet then we aren't showing and we can just return		
			if(!this._alreadyInitialized){
				return;
			}

			if(this._fadeIn.status() == "playing"){
				this._fadeIn.stop();
			}
			this._fadeOut.play();

			if (this._scrollConnected){
				this._scrollConnected = false;
			}
			dojo.forEach(this._modalconnects, dojo.disconnect);
			this._modalconnects = [];
			if(this.refocus){
				this.connect(this._fadeOut,"onEnd",dojo.hitch(dijit,"focus",this._savedFocus));
			}
			if(this._relativePosition){
				delete this._relativePosition;	
			}
			this.open = false;
		},

		layout: function() {
			// summary: Position the Dialog and the underlay
			if(this.domNode.style.visibility != "hidden"){
				this._underlay.layout();
				this._position(); 
			}
		},
		
		destroy: function(){
			dojo.forEach(this._modalconnects, dojo.disconnect);
			if(this.refocus && this.open){
				setTimeout(dojo.hitch(dijit,"focus",this._savedFocus), 25);
			}
			this.inherited(arguments);			
		}
	}
);

dojo.declare(
	"dijit.TooltipDialog",
	[dijit.layout.ContentPane, dijit._Templated, dijit.form._FormMixin, dijit._DialogMixin],
	{
		// summary:
		//		Pops up a dialog that appears like a Tooltip
		//
		// title: String
		// 		Description of tooltip dialog (required for a11Y)
		title: "",

		// doLayout: Boolean
		//		Don't change this parameter from the default value.
		//		This ContentPane parameter doesn't make sense for TooltipDialog, since TooltipDialog
		//		is never a child of a layout container, nor can you specify the size of
		//		TooltipDialog in order to control the size of an inner widget. 
		doLayout: false,

		// autofocus: Boolean
		// 		A Toggle to modify the default focus behavior of a Dialog, which
		// 		is to focus on the first dialog element after opening the dialog.
		//		False will disable autofocusing. Default: true
		autofocus: true,

		"class": "dijitTooltipDialog",

		// _firstFocusItem: DomNode
		//		The pointer to the first focusable node in the dialog
		_firstFocusItem:null,
		
		// _lastFocusItem: DomNode
		//		The domNode that had focus before we took it.
		_lastFocusItem: null,

		templateString: null,
		templatePath: dojo.moduleUrl("dijit", "templates/TooltipDialog.html"),

		postCreate: function(){
			this.inherited(arguments);
			this.connect(this.containerNode, "onkeypress", "_onKey");
			this.containerNode.title = this.title;
		},

		orient: function(/*DomNode*/ node, /*String*/ aroundCorner, /*String*/ corner){
			// summary: configure widget to be displayed in given position relative to the button
			this.domNode.className = this["class"] +" dijitTooltipAB"+(corner.charAt(1)=='L'?"Left":"Right")+" dijitTooltip"+(corner.charAt(0)=='T' ? "Below" : "Above");
		},

		onOpen: function(/*Object*/ pos){
			// summary: called when dialog is displayed
		
			this.orient(this.domNode,pos.aroundCorner, pos.corner);
			this._loadCheck(); // lazy load trigger
			
			if(this.autofocus){
				this._getFocusItems(this.containerNode);
				dijit.focus(this._firstFocusItem);
			}
		},
		
		_onKey: function(/*Event*/ evt){
			// summary: keep keyboard focus in dialog; close dialog on escape key
			var node = evt.target;
			var dk = dojo.keys;
			if (evt.charOrCode === dk.TAB){
				this._getFocusItems(this.containerNode);
			}
			var singleFocusItem = (this._firstFocusItem == this._lastFocusItem);
			if(evt.charOrCode == dk.ESCAPE){
				this.onCancel();
				dojo.stopEvent(evt);
			}else if(node == this._firstFocusItem && evt.shiftKey && evt.charOrCode === dk.TAB){
				if(!singleFocusItem){
					dijit.focus(this._lastFocusItem); // send focus to last item in dialog
				}
				dojo.stopEvent(evt);
			}else if(node == this._lastFocusItem && evt.charOrCode === dk.TAB && !evt.shiftKey){
				if(!singleFocusItem){
					dijit.focus(this._firstFocusItem); // send focus to first item in dialog
				}
				dojo.stopEvent(evt);
			}else if(evt.charOrCode === dk.TAB){
				// we want the browser's default tab handling to move focus
				// but we don't want the tab to propagate upwards
				evt.stopPropagation();
			}
		}
	}	
);

