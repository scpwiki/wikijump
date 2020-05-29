dojo.provide("dojox.charting.plot2d.Bubble");

dojo.require("dojox.charting.plot2d.Base");
dojo.require("dojox.lang.functional");

(function(){
	var df = dojox.lang.functional, du = dojox.lang.utils,
		dc = dojox.charting.plot2d.common,
		purgeGroup = df.lambda("item.purgeGroup()");

	dojo.declare("dojox.charting.plot2d.Bubble", dojox.charting.plot2d.Base, {
		defaultParams: {
			hAxis: "x",		// use a horizontal axis named "x"
			vAxis: "y"		// use a vertical axis named "y"
		},
		optionalParams: {},	// no optional parameters

		constructor: function(chart, kwArgs){
			this.opt = dojo.clone(this.defaultParams);
			du.updateWithObject(this.opt, kwArgs);
			this.series = [];
			this.hAxis = this.opt.hAxis;
			this.vAxis = this.opt.vAxis;
		},
		
		calculateAxes: function(dim){
			this._calc(dim, dc.collectSimpleStats(this.series));
			return this;
		},

		//	override the render so that we are plotting only circles.
		render: function(dim, offsets){
			this.dirty = this.isDirty();
			if(this.dirty){
				dojo.forEach(this.series, purgeGroup);
				this.cleanGroup();
				var s = this.group;
				df.forEachRev(this.series, function(item){ item.cleanGroup(s); });
			}
		
			var t = this.chart.theme, stroke, outline, color, shadowStroke, shadowColor,
				ht = this._hScaler.scaler.getTransformerFromModel(this._hScaler),
				vt = this._vScaler.scaler.getTransformerFromModel(this._vScaler),
				events = this.events();

			this.resetEvents();

			for(var i = this.series.length - 1; i >= 0; --i){
				var run = this.series[i];
				if(!this.dirty && !run.dirty){ continue; }
				run.cleanGroup();
				if(!run.data.length){
					run.dirty = false;
					continue;
				}

				if(typeof run.data[0] == "number"){
					console.warn("dojox.charting.plot2d.Bubble: the data in the following series cannot be rendered as a bubble chart; ", run);
					continue;
				}
				
				var s = run.group,
					points = dojo.map(run.data, function(v, i){
						return {
							x: ht(v.x) + offsets.l,
							y: dim.height - offsets.b - vt(v.y),
							radius: this._vScaler.bounds.scale * (v.size / 2)
						};
					}, this);

				if(run.fill){
					color = run.fill;
				}else if(run.stroke){
					color = run.stroke;
				}else{
					color = run.dyn.color = new dojo.Color(t.next("color"));
				}
				run.dyn.fill = color;

				stroke = run.dyn.stroke = run.stroke ? dc.makeStroke(run.stroke) : dc.augmentStroke(t.series.stroke, color);

				var frontCircles = null, outlineCircles = null, shadowCircles = null;

				// make shadows if needed
				if(this.opt.shadows && stroke){
					var sh = this.opt.shadows, shadowColor = new dojo.Color([0, 0, 0, 0.2]),
						shadowStroke = dojo.clone(outline ? outline : stroke);
					shadowStroke.color = shadowColor;
					shadowStroke.width += sh.dw ? sh.dw : 0;
					run.dyn.shadow = shadowStroke;
					shadowMarkers = dojo.map(points, function(item){
						var sh = this.opt.shadows;
						return s.createCircle({
							cx: item.x + sh.dx, cy: item.y + sh.dy, r: item.radius
						}).setStroke(shadowStroke).setFill(shadowColor);
					}, this);
				}

				// make outlines if needed
				if(run.outline || t.series.outline){
					outline = dc.makeStroke(run.outline ? run.outline : t.series.outline);
					outline.width = 2 * outline.width + stroke.width;
					run.dyn.outline = outline;
					outlineCircles = dojo.map(points, function(item){
						s.createCircle({ cx: item.x, cy: item.y, r: item.radius }).setStroke(outline);
					}, this);
				}

				//	run through the data and add the circles.
				frontCircles = dojo.map(points, function(item){
					return s.createCircle({ cx: item.x, cy: item.y, r: item.radius }).setStroke(stroke).setFill(color);
				}, this);
				
				if(events){
					dojo.forEach(frontCircles, function(s, i){
						var o = {
							element: "circle",
							index:   i,
							run:     run,
							plot:    this,
							hAxis:   this.hAxis || null,
							vAxis:   this.vAxis || null,
							shape:   s,
							outline: outlineCircles && outlineCircles[i] || null,
							shadow:  shadowCircles && shadowCircles[i] || null,
							x:       run.data[i].x,
							y:       run.data[i].y,
							r:       run.data[i].size / 2,
							cx:      points[i].x,
							cy:      points[i].y,
							cr:      points[i].radius
						};
						this._connectEvents(s, o);
					}, this);
				}
				
				run.dirty = false;
			}
			this.dirty = false;
			return this;
		}
	});
})();
