dojo.provide('dojox.widget.Dialog');
dojo.experimental('dojox.widget.Dialog');

dojo.require('dijit.Dialog');
dojo.require('dojox.fx');

dojo.declare('dojox.widget.Dialog', 
	dijit.Dialog, 
	{
	// summary: A Lightbox-like Modal-dialog for HTML Content
	//
	// description:
	//		An HTML 
	templatePath: dojo.moduleUrl('dojox.widget','Dialog/Dialog.html'),
	
	// fixedSize: Boolean
	//		If false, fix the size of the dialog to the Viewport based on 
	//		viewportPadding value rather than the calculated or natural 
	//		stlye. If true, base the size on a passed dimension attribute.
	//		Eitherway, the viewportPadding value is used if the the content
	//		extends beyond the viewport size for whatever reason.
	fixedSize: false,
	
	// viewportPadding: Integer
	//		If fixedSize="true", this is the value  or used when fixed="false" and dimensions exceed) to use
	//		 
	viewportPadding: 35,
	
	// dimensions: Array
	//		A two-element array of [widht,height] to animate the Dialog to. Defaults to [300,300]
	dimensions: null, 
	
	// easing: Function?|String?
	//		An easing function to apply to the sizing animation. 
	easing: null,
	
	// sizeDuration: Integer
	//		Time (in ms) to use in the Animation for sizing. 
	sizeDuration: dijit._defaultDuration,
	
	// sizeMethod: String
	// 		To be passed to dojox.fx.sizeTo, one of "chain" or "combine" to effect
	//		the animation sequence.
	sizeMethod: "chain",
	
	// showTitle: Boolean
	//		Toogle to show or hide the Title area. Can only be set at startup.
	showTitle: false,
	
	// draggable: Boolean
	//		Make the pane draggable. Differs from dijit.Dialog by setting default to false
	draggable: false, // simply over-ride the default from dijit.Dialog 
	
	constructor: function(props, node){
		this.easing = props.easing || dojo._defaultEasing; 
		this.dimensions = props.dimensions || [300, 300];
	},
	
	_setup: function(){
		// summary: Piggyback on dijit.Dialog's _setup for load-time options, deferred to 
		//		
		this.inherited(arguments);
		if(!this._alreadyInitialized){
			// FIXME: should this be optional, too?
			this.connect(this._underlay.domNode,"onclick","onCancel");
			
			this._navIn = dojo.fadeIn({ node: this.closeButtonNode });
			this._navOut = dojo.fadeOut({ node: this.closeButtonNode }); 
			if(!this.showTitle){
				dojo.addClass(this.domNode,"dojoxDialogNoTitle");
			}
		}	
	},
	
	layout: function(e){
		
		this._setSize();
		this.inherited(arguments);
	},
	
	_setSize: function(){
		// summary: cache and set our desired end position 
		this._vp = dijit.getViewport();
		var tc = this.containerNode;
		var fixed = this.fixedSize;
		this._displaysize = {
			w: fixed ? tc.scrollWidth : this.dimensions[0],
			h: fixed ? tc.scrollHeight : this.dimensions[1]
		};
	},
	
	show: function(){
		
		this._setSize();
		dojo.style(this.closeButtonNode,"opacity", 0);
		dojo.style(this.domNode, {
			overflow: "hidden",
			opacity: 0,
			width: "1px",
			height: "1px"
		});
		dojo.style(this.containerNode, {
			opacity: 0,
			overflow: "hidden"
		});	
		
		this.inherited(arguments);

		this._modalconnects.push(dojo.connect(this.domNode,"onmouseenter",this,"_handleNav"));
		this._modalconnects.push(dojo.connect(this.domNode,"onmouseleave",this,"_handleNav"));
		
	},
	
	_handleNav: function(e){
		// summary: Handle's showing or hiding the close icon

		var navou = "_navOut"; 
		var navin = "_navIn";

		var animou = (e.type == "mouseout" ? navin : navou);
		var animin = (e.type == "mouseout" ? navou : navin);
		
		this[animou].stop();
		this[animin].play();
		
	},
	
	// an experiment in a quicksilver-like hide. too choppy for me.
	/*
	hide: function(){
		// summary: Hide the dialog

		// if we haven't been initialized yet then we aren't showing and we can just return		
		if(!this._alreadyInitialized){
			return;
		}

		this._fadeIn && this._fadeIn.stop();

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
		
		dojox.fx.sizeTo({ 
			node: this.domNode,
			duration:this.sizeDuration || this.duration,
			width: this._vp.w - 1,
			height: 5,
			onBegin: dojo.hitch(this,function(){
				this._fadeOut.play(this.sizeDuration / 2);
			})
		}).play();
		
		this.open = false;
	}, */

	_position: function(){
		
		if(this._sizing){
			this._sizing.stop();	
			this.disconnect(this._sizingConnect);
		}
		
		this.inherited(arguments);
		
		if(!this.open){ dojo.style(this.containerNode, "opacity", 0); }
		var pad = this.viewportPadding * 2; 
		
		var props = {
			node: this.domNode,
			duration: this.sizeDuration || dijit._defaultDuration,
			easing: this.easing,
			method: this.sizeMethod
		};

		var ds = this._displaysize;
		props['width'] = ds.w = (ds.w + pad >= this._vp.w || this.fixedSize) 
			? this._vp.w - pad : ds.w;
			
		props['height'] = ds.h = (ds.h + pad >= this._vp.h || this.fixedSize) 
			? this._vp.h - pad : ds.h;

		this._sizing = dojox.fx.sizeTo(props);
		this._sizingConnect = this.connect(this._sizing,"onEnd","_showContent");
		this._sizing.play();

	},

	_showContent: function(e){
		// summary: Show the inner container after sizing animation

		var container = this.containerNode;
		dojo.style(this.domNode,"overflow","visible");
		dojo.style(container, {
			height: this._displaysize.h + "px",
			width: this._displaysize.w + "px",
			overflow:"auto"
		});
		dojo.anim(container, { opacity:1 });
	}
	
});