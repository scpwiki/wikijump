dojo.provide("dojox.charting.Theme");
dojo.require("dojox.color");
dojo.require("dojox.color.Palette");

(function(){
	var dxc=dojox.charting;
	//	TODO: Legend information

	dxc.Theme = function(/*Object?*/ kwArgs){
		kwArgs=kwArgs||{};
		var def = dxc.Theme._def;
		dojo.forEach(["chart", "plotarea", "axis", "series", "marker"], function(n){
			this[n] = dojo.mixin(dojo.clone(def[n]), kwArgs[n]||{});
		}, this);
		this.markers = dojo.mixin(dojo.clone(dxc.Theme.Markers), kwArgs.markers||{});
		this.colors = [];
		this.antiAlias = ("antiAlias" in kwArgs)?kwArgs.antiAlias:true;
		this.assignColors = ("assignColors" in kwArgs)?kwArgs.assignColors:true;
		this.assignMarkers = ("assignMarkers" in kwArgs)?kwArgs.assignMarkers:true;

		//	push the colors, use _def colors if none passed.
		kwArgs.colors = kwArgs.colors||def.colors;
		dojo.forEach(kwArgs.colors, function(item){ 
			this.colors.push(item); 
		}, this);

		//	private variables for color and marker indexing
		this._current = { color:0, marker: 0 };
		this._markers = [];
		this._buildMarkerArray();
	};

	//	"static" fields
	//	default markers.
	//	A marker is defined by an SVG path segment; it should be defined as
	//		relative motion, and with the assumption that the path segment
	//		will be moved to the value point (i.e prepend Mx,y)
	dxc.Theme.Markers={
		CIRCLE:		"m-3,0 c0,-4 6,-4 6,0 m-6,0 c0,4 6,4 6,0", 
		SQUARE:		"m-3,-3 l0,6 6,0 0,-6 z", 
		DIAMOND:	"m0,-3 l3,3 -3,3 -3,-3 z", 
		CROSS:		"m0,-3 l0,6 m-3,-3 l6,0", 
		X:			"m-3,-3 l6,6 m0,-6 l-6,6", 
		TRIANGLE:	"m-3,3 l3,-6 3,6 z", 
		TRIANGLE_INVERTED:"m-3,-3 l3,6 3,-6 z"
	};
	dxc.Theme._def={
		//	all objects are structs used directly in dojox.gfx
		chart:{ 
			stroke:null,
			fill: "white"
		},
		plotarea:{ 
			stroke:null,
			fill: "white"
		},
		//	TODO: label rotation on axis
		axis:{
			stroke:	{ //	the axis itself
				color:"#333",
				width:1
			},
			/*
			line:	{ //	in the future can be used for gridlines
				color:"#ccc",
				width:1,
				style:"Dot",
				cap:"round"
			},
			*/
			majorTick:	{ //	major ticks on axis, and used for major gridlines
				color:"#666",
				width:1, 
				length:6, 
				position:"center"
			},
			minorTick:	{ //	minor ticks on axis, and used for minor gridlines
				color:"#666", 
				width:0.8, 
				length:3, 
				position:"center"
			},	
			font: "normal normal normal 7pt Tahoma", //	labels on axis
			fontColor:"#333"						//	color of labels
		},
		series:{
			outline: {width: 0.1, color: "#ccc"},							//	line or outline
			stroke: {width: 1.5, color: "#333"},							//	line or outline
			fill: "#ccc",												//	fill, if appropriate
			font: "normal normal normal 7pt Tahoma",					//	if there's a label
			fontColor: "#000"											// 	color of labels
		},
		marker:{	//	any markers on a series.
			stroke: {width:1},											//	stroke or outline
			fill: "#333",												//	fill if needed
			font: "normal normal normal 7pt Tahoma",					//	label
			fontColor: "#000"
		},
		colors:[ "#54544c","#858e94","#6e767a","#948585","#474747" ]
	};
	
	//	prototype methods
	dojo.extend(dxc.Theme, {
		defineColors: function(obj){
			//	summary:
			//		Generate a set of colors for the theme based on keyword
			//		arguments
			var kwArgs=obj||{};

			//	note that we've changed the default number from 32 to 4 colors
			//	are cycled anyways.
			var c=[], n=kwArgs.num||5;	//	the number of colors to generate
			if(kwArgs.colors){
				//	we have an array of colors predefined, so fix for the number of series.
				var l=kwArgs.colors.length;
				for(var i=0; i<n; i++){
					c.push(kwArgs.colors[i%l]);
				}
				this.colors=c;
			}else if(kwArgs.hue){
				//	single hue, generate a set based on brightness
				var s=kwArgs.saturation||100;	//	saturation
				var st=kwArgs.low||30;
				var end=kwArgs.high||90;
				//	we'd like it to be a little on the darker side.
				var l=(end+st)/2;

				//	alternately, use "shades"
				this.colors = dojox.color.Palette.generate(
					dojox.color.fromHsv(kwArgs.hue, s, l), "monochromatic"
				).colors;
			}else if(kwArgs.generator){
				//	pass a base color and the name of a generator
				this.colors=dojox.color.Palette.generate(kwArgs.base, kwArgs.generator).colors;
			}
		},
	
		_buildMarkerArray: function(){
			this._markers = [];
			for(var p in this.markers){ this._markers.push(this.markers[p]); }
			//	reset the position
			this._current.marker=0;
		},

		_clone: function(){
			//	summary:
			//		Return a clone of this theme, with the position vars reset to 0.
			return new dxc.Theme({
				chart: this.chart,
				plotarea: this.plotarea,
				axis: this.axis,
				series: this.series,
				marker: this.marker,
				antiAlias: this.antiAlias,
				assignColors: this.assignColors,
				assignMarkers: this.assigneMarkers,
				colors: dojo.clone(this.colors)
			});
		},

		addMarker:function(/*String*/ name, /*String*/ segment){
			//	summary:
			//		Add a custom marker to this theme.
			//	example:
			//	|	myTheme.addMarker("Ellipse", foo);
			this.markers[name]=segment;
			this._buildMarkerArray();
		},
		setMarkers:function(/*Object*/ obj){
			//	summary:
			//		Set all the markers of this theme at once.  obj should be a
			//		dictionary of keys and path segments.
			//
			//	example:
			//	|	myTheme.setMarkers({ "CIRCLE": foo });
			this.markers=obj;
			this._buildMarkerArray();
		},

		next: function(/*String?*/ type){
			//	summary:
			//		get either the next color or the next marker, depending on
			//		what was passed. If type is not passed, it assumes color.
			//	type:
			//		Optional. One of either "color" or "marker". Defaults to
			//		"color".
			//	example:
			//	|	var color = myTheme.next();
			//	|	var color = myTheme.next("color");
			//	|	var marker = myTheme.next("marker");
			if(type == "marker"){
				return this._markers[ this._current.marker++ % this._markers.length ];
			}else{
				return this.colors[ this._current.color++ % this.colors.length ];
			}
		},
		clear: function(){
			// summary:
			//		resets both marker and color counters back to the start.
			//		Subsequent calls to `next` will retrievie the first value
			//		of each depending on the passed type.
			this._current = {color: 0, marker: 0};
		}
	});
})();
