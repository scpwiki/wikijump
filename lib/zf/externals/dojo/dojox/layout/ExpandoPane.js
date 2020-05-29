dojo.provide("dojox.layout.ExpandoPane");
dojo.experimental("dojox.layout.ExpandoPane"); // just to show it can be done?

dojo.require("dijit.layout.ContentPane");
dojo.require("dijit._Templated");
dojo.require("dijit._Container");

dojo.declare("dojox.layout.ExpandoPane",
	[dijit.layout.ContentPane, dijit._Templated, dijit._Contained],
	{
	// summary: An experimental expando-pane for dijit.layout.BorderContainer
	//
	// description:
	//		Works just like a ContentPane inside of a borderContainer. Will expand/collapse on
	//		command, and supports having Layout Children as direct descendants
	//		via a custom "attachParent" attribute on the child.

	maxHeight: "",
	maxWidth: "",
	splitter: "",

	templatePath: dojo.moduleUrl("dojox.layout","resources/ExpandoPane.html"),

	_showing: true,

	// easeOut: String|Function
	//		easing function used to hide pane
	easeOut: "dojo._DefaultEasing",
	
	// easeIn: String|Function
	//		easing function use to show pane
	easeIn: "dojo._DefaultEasing",
	
	// duration: Integer
	//		duration to run show/hide animations
	duration: 420,
	
	startExpanded: true, 

	baseClass: "dijitExpandoPane",

	postCreate: function(){
		this.inherited(arguments);
		this._animConnects = [];

		this._isHorizontal = true;
		
		if(dojo.isString(this.easeOut)){
			this.easeOut = dojo.getObject(this.easeOut);
		}
		if(dojo.isString(this.easeIn)){
			this.easeIn = dojo.getObject(this.easeIn); 
		}
	
		var thisClass = "";
		if(this.region){
			// FIXME: add suport for alternate region types?
			switch(this.region){
				case "right" :
					thisClass = "Right";
					break;
				case "left" :
					thisClass = "Left";
					break;
				case "top" :
					thisClass = "Top";
					break;
				case "bottom" :
					thisClass = "Bottom"; 
					break;
			}
			dojo.addClass(this.domNode, "dojoxExpando" + thisClass);
			this._isHorizontal = /top|bottom/.test(this.region);
		}
		dojo.style(this.domNode, {
			overflow: "hidden",
			padding:0
		});
	},
	
	startup: function(){
		this.inherited(arguments);
		
		this._container = this.getParent();
		this._closedSize = this._titleHeight = dojo.marginBox/*_getBorderBox*/(this.titleWrapper).h;
		
		if(this.splitter){
			// find our splitter and tie into it's drag logic
			var myid = this.id;
			dijit.registry.filter(function(w){
				return w && w.child && w.child.id == myid;
			}).forEach(dojo.hitch(this,function(w){
				this.connect(w,"_stopDrag","_afterResize");
			}));
		}
		
		this._currentSize = dojo.marginBox(this.domNode);
		this._showSize = this._currentSize[(this._isHorizontal ? "h" : "w")];
		this._setupAnims();

		if(this.startExpanded){
			this._showing = true;
		}else{
			this._hideWrapper();
			this._hideAnim.gotoPercent(99,true);
		}
	},
	
	_afterResize: function(e){
		var tmp = this._currentSize;
		this._currentSize = dojo.marginBox(this.domNode);
		var n = this._currentSize[(this._isHorizontal ? "h" : "w")] 
		if(n > this._titleHeight){
			if(!this._showing){	
				this._showing = !this._showing; 
				this._showEnd();
			}
			this._showSize = n;
			this._setupAnims();
		}else{
			this._showSize = tmp[(this._isHorizontal ? "h" : "w")];
			this._showing = false;
			this._hideWrapper();
			this._hideAnim.gotoPercent(89,true);
		}

	},
	
	_setupAnims: function(){
		// summary: Create the show and hide animations
		dojo.forEach(this._animConnects, dojo.disconnect);
		
		var _common = {
			node:this.domNode,
			duration:this.duration
		};

		var isHorizontal = this._isHorizontal;
		var showProps = {};
		var hideProps = {};

		var dimension = isHorizontal ? "height" : "width";
		showProps[dimension] = { 
			end: this._showSize, 
			unit:"px" 
		};
		hideProps[dimension] = { 
			end: this._closedSize, 
			unit:"px"
		};

		this._showAnim = dojo.animateProperty(dojo.mixin(_common,{
			easing:this.easeIn,
			properties: showProps 
		}));
		this._hideAnim = dojo.animateProperty(dojo.mixin(_common,{
			easing:this.easeOut,
			properties: hideProps
		}));

		this._animConnects = [
			dojo.connect(this._showAnim, "onEnd", this, "_showEnd"),
			dojo.connect(this._hideAnim, "onEnd", this, "_hideEnd")
		];
	},
	
	toggle: function(){
		// summary: Toggle this pane's visibility
		if(this._showing){
			this._hideWrapper();
			this._showAnim && this._showAnim.stop();
			this._hideAnim.play();
		}else{
			this._hideAnim && this._hideAnim.stop();
			this._showAnim.play();
		}
		this._showing = !this._showing;
	},
	
	_hideWrapper: function(){
		// summary: Set the Expando state to "closed"
		dojo.addClass(this.domNode, "dojoxExpandoClosed");
		dojo.style(this.cwrapper,{
			visibility: "hidden",
			opacity: "0",
			overflow: "hidden"
		});
	},
	
	_showEnd: function(){
		// summary: Common animation onEnd code - "unclose"
		dojo.style(this.cwrapper, { opacity: 0, visibility:"visible" });		
		dojo.fadeIn({ node:this.cwrapper, duration:227 }).play(1);
		dojo.removeClass(this.domNode, "dojoxExpandoClosed");
		setTimeout(dojo.hitch(this._container, "layout"), 15);
	},
	
	_hideEnd: function(){
		setTimeout(dojo.hitch(this._container, "layout"), 15);
	},
	
	resize: function(){
		// summary: we aren't a layout widget, but need to act like one:
		var size = dojo.marginBox(this.domNode),
			h = size.h - this._titleHeight;
			
		dojo.style(this.containerNode, "height", h + "px");
		this.inherited(arguments);
	},
	
	_trap: function(e){
		dojo.stopEvent(e);
	}

});
