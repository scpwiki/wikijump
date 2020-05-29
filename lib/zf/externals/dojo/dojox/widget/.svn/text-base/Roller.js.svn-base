dojo.provide("dojox.widget.Roller");
dojo.require("dijit._Widget");

dojo.declare("dojox.widget.Roller", dijit._Widget, {
	// summary: A simple widget to take an unorder-list of Text and roll through them
	// 
	// description: 
	//		The Roller widget takes an unordered-list of items, and converts
	//		them to a single-area (the size of one list-item, however you so choose
	//		to style it) and loops continually, fading between items. 
	//		
	//		In it's current state, it requires it be created from an unordered (or ordered)
	//		list. 
	//
	//		You can manipulate the `items` array at any point during the cycle with
	//		standard array manipulation techniques.
	//
	//	example: 
	//	|	// create a scroller from a unorderlist with id="lister"
	//  |	var thinger = new dojox.widget.Roller.Roller({},"lister");
	//
	//	example:
	//	|	// create a scroller from a fixed array:
	//	|	new dojox.widget.Roller({ items:["one","two","three"] });
	//
	//	example:
	//	|	// add an item:
	//	|	dijit.byId("roller").items.push("I am a new Label");
	//
	//  example: 
	//	|	// stop a roller from rolling:
	//	|	dijit.byId("roller").stop();
	//
	// delay: Integer
	//		Interval between rolls
	delay: 2000,

	// autoStart: Boolean
	//		Toggle to control starup behavior. Call .start() manually
	//		if set to `false`
	autoStart: true,
	
/*=====
	// items: Array
	//		If populated prior to instantiation, is used as the Items over the children
	items: []
=====*/	

	postCreate: function(){

		// add some instance vars:
		if(!this["items"]){ 
			this.items = [];
		}
		this._idx = -1;
		
		dojo.addClass(this.domNode,"dojoxRoller");
		
		// find all the items in this list, and popuplate 
		dojo.query("li", this.domNode).forEach(function(item){
			this.items.push(item.innerHTML);
			dojo._destroyElement(item);
		}, this);

		// add back a default item
		this._roller = dojo.doc.createElement('li');
		this.domNode.appendChild(this._roller);
		
		// stub out animation creation (for overloading maybe later)
		this.makeAnims();
		
		// and start, if true:
		if(this.autoStart){ this.start(); }
		
	},

	makeAnims: function(){
		// summary: Animation creator function. Need to create an 'in' and 'out'
		// 		_Animation stored in _anim Object, which the rest of the widget
		//		will reuse. 
		var n = this.domNode;
		dojo.mixin(this, {
			_anim: {
				"in": dojo.fadeIn({ node:n, duration: 400 }),
				"out": dojo.fadeOut({ node:n, duration: 275 })
			}
		});
		this._setupConnects();
		
	},
	
	_setupConnects: function(){
		// summary: setup the loop connection logic
		var anim = this._anim;

		this.connect(anim["out"], "onEnd", function(){
			// onEnd of the `out` animation, select the next items and play `in` animation
			this._set(this._idx + 1);
			anim["in"].play(15);
		});
		
		this.connect(anim["in"], "onEnd", function(){
			// onEnd of the `in` animation, call `start` again after some delay:
			this._timeout = setTimeout(dojo.hitch(this, "_run"), this.delay);
		});
	},
	
	start: function(){
		// summary: Starts to Roller looping
		if(!this.rolling){
			this.rolling = true;
			this._run();
		}
	},
	
	_run: function(){
		this._anim["out"].gotoPercent(0, true);
	},

	stop: function(){
		// summary: Stops the Roller from looping anymore.
		this.rolling = false;

		var m = this._anim, 
			t = this._timeout;

		if(t){ clearTimeout(t); }
		m["in"].stop();
		m["out"].stop();
	},
	
	_set: function(i){
		// summary: Set the Roller to some passed index. If beyond range, go to first.
		var l = this.items.length - 1;
		if(i < 0){ i = l; }
		if(i > l){ i = 0; }
		this._roller.innerHTML = this.items[i] || "error!";
		this._idx = i;
	}

});

dojo.declare("dojox.widget.RollerSlide", dojox.widget.Roller, {
	// summary: An add-on to the Roller to modify animations. This produces 
	//		a slide-from-bottom like effect
	
	makeAnims: function(){
		// summary: Animation creator function. Need to create an 'in' and 'out'
		// 		_Animation stored in _anim Object, which the rest of the widget
		//		will reuse.
		var n = this.domNode;

		var pos = "position";
		dojo.style(n, pos, "relative");
		dojo.style(this._roller, pos, "absolute");

		var props = {
			top: { end:0, start: 25 },
			opacity:1
		};

		dojo.mixin(this, {
			_anim: {
				
				"in": dojo.animateProperty({ 
					node: n, 
					duration: 400,
					properties: props
				}),
				
				"out": dojo.fadeOut({ node: n, duration: 175 })
			}
		});
		// don't forget to do this in the class. override if necessary.
		this._setupConnects();
	}
	
});
