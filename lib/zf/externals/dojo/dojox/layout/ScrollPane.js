dojo.provide("dojox.layout.ScrollPane");
dojo.experimental("dojox.layout.ScrollPane");

dojo.require("dijit.layout._LayoutWidget");
dojo.require("dijit._Templated");

dojo.declare("dojox.layout.ScrollPane",
	[dijit.layout._LayoutWidget,dijit._Templated],
	{
	// summary: A pane that "scrolls" its content based on the mouse poisition inside
	//
	// description:
	//		A sizable container that takes it's content's natural size and creates
	//		a scroll effect based on the relative mouse position. It is an interesting
	//		way to display lists of data, or blocks of content, within a confined
	//		space.
	//
	// 		Horizontal scrolling is supported. Combination scrolling is not.
	//
	//		FIXME: need to adust the _line somehow, it stops scrolling
	//		
	// example:
	// |	<div dojoType="dojox.layout.ScrollPane" style="width:150px height:300px;">
	// |		<!-- any height content -->
	// |	</div>
	//
	// _line: dojo._Line
	// 		storage for our top and bottom most scrollpoints
	_line: null,
	
	// _lo: the height of the visible pane
	_lo: null,
	
	_offset: 15,
	
	// orientation: String
	//		either "horizontal" or "vertical" for scroll orientation. 
	orientation: "vertical",
	
	templatePath: dojo.moduleUrl("dojox.layout","resources/ScrollPane.html"),
	
	layout: function(){
		// summary: calculates required sizes. Call this if you add/remove content manually, or reload the content.

		var dir = this._dir,
			vert = this._vertical,
			val = this.containerNode[(vert ? "scrollHeight" : "scrollWidth")];
		
		dojo.style(this.wrapper, dir, this.domNode.style[dir]);
		
		this._lo = dojo.coords(this.wrapper, true);
		this._size = Math.max(0, val - this._lo[(vert ? "h" : "w")]);
		this._line = new dojo._Line(0 - this._offset, this._size + (this._offset * 2));
	
		// share a relative position w the scroll offset via a line
		var u = this._lo[(vert ? "h" : "w")],
			r = Math.min(1, u / val), // ratio
			s = u * r, // size
			c = Math.floor(u - (u * r)); // center
			  
		this._helpLine = new dojo._Line(0, c);
	
		// size the helper
		dojo.style(this.helper, dir, Math.floor(s) + "px");
		
	},
	
	postCreate: function(){
		this.inherited(arguments);
	
		// for the helper
		this._showAnim = dojo._fade({ node:this.helper, end:0.5, duration:350 });
		this._hideAnim = dojo.fadeOut({ node:this.helper, duration: 750 });
	
		// orientation helper
		this._vertical = (this.orientation == "vertical");
		if(!this._vertical){
			dojo.addClass(this.containerNode,"dijitInline");
			this._dir = "width";
			this._edge = "left";
		}else{
			this._dir = "height";
			this._edge = "top";
		}
		
		this._hideAnim.play();
		dojo.style(this.wrapper,"overflow","hidden");
	
	},	
	
	_set: function(/* Float */n){
		// summary: set the pane's scroll offset, and position the virtual scroll helper 
		this.wrapper[(this._vertical ? "scrollTop" : "scrollLeft")] = Math.floor(this._line.getValue(n));
		dojo.style(this.helper, this._edge, Math.floor(this._helpLine.getValue(n)) + "px");    
	},
	
	_calc: function(/* Event */e){
		// summary: calculate the relative offset of the cursor over the node, and call _set
		this._set(this._vertical ? 
			((e.pageY - this._lo.y) / this._lo.h) :
			((e.pageX - this._lo.x) / this._lo.w)
		);
	},
	
	_enter: function(e){
		if(this._hideAnim && this._hideAnim.status() == "playing"){ 
			this._hideAnim.stop(); 
		}
		this._showAnim.play();
	},
	
	_leave: function(e){
		this._hideAnim.play();
	}
    
});
