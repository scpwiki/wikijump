dojo.provide("demos.mojo.src");

// our core requirements:
dojo.require("dojo.dnd.Moveable");
dojo.require("dojo.fx");
dojo.require("dojo.fx.easing");
dojo.require("dojox.widget.Roller");

// our custom code:
dojo.require("demos.mojo.src.drop"); // gravity code
dojo.require("demos.mojo.src.download"); // download link code

(function(){ 
		
	var nodes, style = dojo.style;
	
	dojo.addOnLoad(function(){
		
		nodes = dojo.query("#container > div");
		// iterate over each div in the container
		nodes.forEach(function(n){
			// hide the node, first thing, and undo native-css hiding:
			style(n, { opacity:0, visibility:"visible" });

			// the drag handle will be the h1 element in this div
			var handle = dojo.query("h1", n)[0];
			new dojo.dnd.Moveable(n, { handle: handle });

			// there is really only one image in here though:
			dojo.query("img", n).forEach(function(img){
				style(img,{
					width:"1px", height:"1px",
					top:"155px", left:"155px;"
				});
				if(dojo.isIE){
					// no png's for ie users
					img.src = "images/shot3.gif";
				}
			});
		});
		
		// dojo.fx.combine takes an array of animations:
		var _anims = [];
		var _delay = 1200;
		
		nodes.forEach(function(n){
			// fade in the node, delayed 500ms
			_anims.push(dojo.fadeIn({
				duration:850,
				node: n, delay: _delay + 1200,
				properties: {
					paddingTop: {
						start:155, end:1, unit:"px"
					},
					fontSize:{
						start:0.1, end:16, unit:"px"
					}
				}
			}))
			
			dojo.query("img", n).forEach(function(img){
				_anims.push(dojo.animateProperty({
					duration:450,
					delay: _delay + 1000,
					node: img,
					properties: {
						width: 310, //{ end:310, unit:"px" },
						height: 310, //{ end:310, unit:"px" },
						top: 0, // { end:0 }, 
						left: 0 // { end: 0 }
					}
				}));
			});

			_delay += 1500; // step up the delay base just a smidge

		});
		
		// add the header-in-animation to our _anims array
		_anims.push(dojo.animateProperty({
			node: "header",
			properties: {
				top: 5, //{ end: 5, unit:"px" },
				left: 5 // { end: 5, unit:"px" }
			},
			delay: _delay,
			duration: 700
		}));
		
		_anims.push(dojo.fadeIn({
			node:"downloadButton",
			duration:400,
			delay:2000,
			beforeBegin: dojo.partial(style, "downloadButton", { 
				opacity:0, visibility:"visible" 
			})
		}));
		
		// combine them all, and play a s single animation (with a
		// setTimeout to give the broswer a second to be sane again)
		var anim = dojo.fx.combine(_anims);
		
		var roller = new dojox.widget.RollerSlide({ delay:5000, autoStart:false },"whyList");
		dojo.connect(anim,"onEnd", roller, "start");

		setTimeout(dojo.hitch(anim,"play"), 15);
		
		var _coords = null;
		var _z = null;
		
		dojo.subscribe("/dnd/move/start",function(e){
			// when drag starts, save the coords of the node we're pulling
			var n = e.node;
			_coords = dojo.coords(n);
			// and "bring to top"
			// and make it partially opaque
			_z = style(n, "zIndex");
			style(n, { zIndex:888, opacity:0.65 });
		});
		
		dojo.subscribe("/dnd/move/stop", function(e){
			// when it ends, reset z-index, opacity, and animate back to spot
			style(e.node, "opacity", 1);
			if(_coords){
				dojo.fx.slideTo({
					node: e.node, // drag node
					top: _coords.t, // tmp top
					left: _coords.l, // tmp left
					easing: dojo.fx.easing.elasticOut,
					duration:950 // ms
				}).play(5); // small delay for performance?
				style(e.node, "zIndex", _z);
			}
		});	

	});

})();
