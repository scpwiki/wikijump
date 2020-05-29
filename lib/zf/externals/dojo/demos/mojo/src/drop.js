dojo.provide("demos.mojo.src.drop");
// adds gravity effect to mojoDemo
dojo.require("dojo.fx");
dojo.require("dojo.fx.easing");
dojo.require("dijit._base.place");
(function(){ 

	var mojo = {};
	var nodes;
	var _coords = [];
	var cb = null;
	
	dojo.mixin(mojo, {
		drop: {
			_calcPositions: function(e){
				// store or return the current calculated positions of each ball
				var c = [];
				nodes.forEach(function(n){
					c.push(dojo.coords(n));
				});

				if(e){
					_coords = c; // with luck maybe in this limited scope, but this is bad.
				}
				return c; // Array
			},
			dropNodes: function(){
				// summary: drop all the nodes using the bounce easing function

				_coords = mojo.drop._calcPositions(); // store positions for later
				// ball is 310px, so the bottom edges are height - 310 roughly.
				var t = dijit.getViewport().h - 310; 
				dojo.style(dojo.body(),"overflow","hidden");

				var _anims = [];
				nodes.forEach(function(n,idx){
					// we want to keep the left: attribute in tact, so pass it along
					var l = _coords[idx].l;
					_anims.push(dojo.fx.slideTo({
						top:t, left:l, node:n,
						duration:1000, 
						easing:dojo.fx.easing.bounceOut
					}));
				});
				// play the _anims as one animation
				dojo.fx.combine(_anims).play();
			},
			floatNodes: function(){
				// summary: reset all the nodes to the orig. positions
				var _anims = [];
				nodes.forEach(function(n,idx){
					// push each slide animation in _anims, based on it's stored coords
					var t = _coords[idx].t;
					var l = _coords[idx].l;
					_anims.push(dojo.fx.slideTo({
						top: t, left:l, node:n,
						duration:500
					}));			
				});
				// play the anim, and set overflow:auto on body
				var _anim = dojo.fx.combine(_anims);
				var con = dojo.connect(_anim,"onEnd",function(){
					dojo.style(dojo.body(), "overflow", "visible"); 
				});
				_anim.play();
			}
		}
	})
	
	var _toggleGravity = function(e){
		// drop or float the nodes based on the state of the checkbox
		mojo.drop[(cb.checked ? "dropNodes" : "floatNodes")]();
	};
	
	dojo.addOnLoad(function(){
		// convenience:
		nodes = dojo.query("#container > div");

		// setup the "gravity toggler"
		cb = dojo.byId("gravity");
		cb.checked = false;
		// FIXME: ie7 fires onchange after blur() ... ugh
		dojo.connect(cb,"onchange",_toggleGravity);

		// just in case, because our nodes are absolutely positioned:
		dojo.connect(window,"onresize",mojo.drop,"_calcPositions");

		// lets make the logo drag/snap-able, too.
		new dojo.dnd.Moveable("logoImg");
		dojo.style("logoImg","cursor","move");
	});

})();
