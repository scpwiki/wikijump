dojo.provide("dojox.charting.axis2d.Default");

dojo.require("dojox.charting.scaler.linear");
dojo.require("dojox.charting.axis2d.common");
dojo.require("dojox.charting.axis2d.Base");

dojo.require("dojo.colors");
dojo.require("dojo.string");
dojo.require("dojox.gfx");
dojo.require("dojox.lang.functional");
dojo.require("dojox.lang.utils");

(function(){
	var dc = dojox.charting,
		df = dojox.lang.functional,
		du = dojox.lang.utils,
		g = dojox.gfx,
		lin = dc.scaler.linear,
		labelGap = 4;	// in pixels

	dojo.declare("dojox.charting.axis2d.Default", dojox.charting.axis2d.Base, {
		 defaultParams: {
			vertical:    false,		// true for vertical axis
			fixUpper:    "none",	// align the upper on ticks: "major", "minor", "micro", "none"
			fixLower:    "none",	// align the lower on ticks: "major", "minor", "micro", "none"
			natural:     false,		// all tick marks should be made on natural numbers
			leftBottom:  true,		// position of the axis, used with "vertical"
			includeZero: false,		// 0 should be included
			fixed:       true,		// all labels are fixed numbers
			majorLabels: true,		// draw major labels
			minorTicks:  true,		// draw minor ticks
			minorLabels: true,		// draw minor labels
			microTicks:  false,		// draw micro ticks
			htmlLabels:  true		// use HTML to draw labels
		},
		optionalParams: {
			min:           0,	// minimal value on this axis
			max:           1,	// maximal value on this axis
			from:          0,	// visible from this value
			to:            1,	// visible to this value
			majorTickStep: 4,	// major tick step
			minorTickStep: 2,	// minor tick step
			microTickStep: 1,	// micro tick step
			labels:        [],	// array of labels for major ticks
								// with corresponding numeric values
								// ordered by values
			// theme components
			stroke:        {},	// stroke for an axis
			majorTick:     {},	// stroke + length for a tick
			minorTick:     {},	// stroke + length for a tick
			font:          "",	// font for labels
			fontColor:     ""	// color for labels as a string
		},

		constructor: function(chart, kwArgs){
			this.opt = dojo.clone(this.defaultParams);
			du.updateWithObject(this.opt, kwArgs);
			du.updateWithPattern(this.opt, kwArgs, this.optionalParams);
		},
		dependOnData: function(){
			return !("min" in this.opt) || !("max" in this.opt);
		},
		clear: function(){
			delete this.scaler;
			delete this.ticks;
			this.dirty = true;
			return this;
		},
		initialized: function(){
			return "scaler" in this && !(this.dirty && this.dependOnData());
		},
		setWindow: function(scale, offset){
			this.scale  = scale;
			this.offset = offset;
			return this.clear();
		},
		getWindowScale: function(){
			return "scale" in this ? this.scale : 1;
		},
		getWindowOffset: function(){
			return "offset" in this ? this.offset : 0;
		},
		calculate: function(min, max, span, labels){
			if(this.initialized()){ return this; }
			this.labels = "labels" in this.opt ? this.opt.labels : labels;
			this.scaler = lin.buildScaler(min, max, span, this.opt);
			if("scale" in this){
				// calculate new range
				this.opt.from = this.scaler.bounds.lower + this.offset;
				this.opt.to   = (this.scaler.bounds.upper - this.scaler.bounds.lower) / this.scale + this.opt.from;
				// make sure that bounds are correct
				if(!isFinite(this.opt.from) || isNaN(this.opt.from) || !isFinite(this.opt.to) || isNaN(this.opt.to) ||
						this.opt.to - this.opt.from >= this.scaler.bounds.upper - this.scaler.bounds.lower){
					// any error --- remove from/to bounds
					delete this.opt.from;
					delete this.opt.to;
					delete this.scale;
					delete this.offset;
				}else{
					// shift the window, if we are out of bounds
					if(this.opt.from < this.scaler.bounds.lower){
						this.opt.to   += this.scaler.bounds.lower - this.opt.from;
						this.opt.from  = this.scaler.bounds.lower;
					}else if(this.opt.to > this.scaler.bounds.upper){
						this.opt.from += this.scaler.bounds.upper - this.opt.to;
						this.opt.to    = this.scaler.bounds.upper;
					}
					// update the offset
					this.offset = this.opt.from - this.scaler.bounds.lower;
				}
				// re-calculate the scaler
				this.scaler = lin.buildScaler(min, max, span, this.opt);
				// cleanup
				if(this.scale == 1 && this.offset == 0){
					delete this.scale;
					delete this.offset;
				}
			}
			var minMinorStep = 0, ta = this.chart.theme.axis,
				taFont = "font" in this.opt ? this.opt.font : ta.font,
				size = taFont ? g.normalizedLength(g.splitFontString(taFont).size) : 0;
			if(this.vertical){
				if(size){
					minMinorStep = size + labelGap;
				}
			}else{
				if(size){
					var labelWidth, i;
					if(this.labels){
						labelWidth = df.foldl(df.map(this.labels, function(label){
							return dojox.gfx._base._getTextBox(label.text, {font: taFont}).w;
						}), "Math.max(a, b)", 0);
					}else{
						var labelLength = Math.ceil(Math.log(Math.max(Math.abs(this.scaler.bounds.from),
								Math.abs(this.scaler.bounds.to))) / Math.LN10),
							t = [];
						if(this.scaler.bounds.from < 0 || this.scaler.bounds.to < 0){ t.push("-"); }
						t.push(dojo.string.rep("9", labelLength));
						var precision = Math.floor(Math.log(this.scaler.bounds.to - this.scaler.bounds.from) / Math.LN10);
						if(precision > 0){
							t.push(".");
							for(i = 0; i < precision; ++i){ t.push("9"); }
						}
						labelWidth = dojox.gfx._base._getTextBox(t.join(""), {font: taFont}).w;
					}
					minMinorStep = labelWidth + labelGap;
				}
			}
			this.scaler.minMinorStep = minMinorStep;
			this.ticks = lin.buildTicks(this.scaler, this.opt);
			return this;
		},
		getScaler: function(){
			return this.scaler;
		},
		getTicks: function(){
			return this.ticks;
		},
		getOffsets: function(){
			var offsets = {l: 0, r: 0, t: 0, b: 0}, labelWidth, a, b, c, d,
				gtb = dojox.gfx._base._getTextBox, gl = dc.scaler.common.getNumericLabel,
				offset = 0, ta = this.chart.theme.axis,
				taFont = "font" in this.opt ? this.opt.font : ta.font,
				taMajorTick = "majorTick" in this.opt ? this.opt.majorTick : ta.majorTick,
				taMinorTick = "minorTick" in this.opt ? this.opt.minorTick : ta.minorTick,
				size = taFont ? g.normalizedLength(g.splitFontString(taFont).size) : 0,
				s = this.scaler;
			if(!s){
				return offsets;
			}
			if(this.vertical){
				if(size){
					if(this.labels){
						labelWidth = df.foldl(df.map(this.labels, function(label){
							return dojox.gfx._base._getTextBox(label.text, {font: taFont}).w;
						}), "Math.max(a, b)", 0);
					}else{
						a = gtb(gl(s.major.start, s.major.prec, this.opt), {font: taFont}).w;
						b = gtb(gl(s.major.start + s.major.count * s.major.tick, s.major.prec, this.opt), {font: taFont}).w;
						c = gtb(gl(s.minor.start, s.minor.prec, this.opt), {font: taFont}).w;
						d = gtb(gl(s.minor.start + s.minor.count * s.minor.tick, s.minor.prec, this.opt), {font: taFont}).w;
						labelWidth = Math.max(a, b, c, d);
					}
					offset = labelWidth + labelGap;
				}
				offset += labelGap + Math.max(taMajorTick.length, taMinorTick.length);
				offsets[this.opt.leftBottom ? "l" : "r"] = offset;
				offsets.t = offsets.b = size / 2;
			}else{
				if(size){
					offset = size + labelGap;
				}
				offset += labelGap + Math.max(taMajorTick.length, taMinorTick.length);
				offsets[this.opt.leftBottom ? "b" : "t"] = offset;
				if(size){
					if(this.labels){
						labelWidth = df.foldl(df.map(this.labels, function(label){
							return dojox.gfx._base._getTextBox(label.text, {font: taFont}).w;
						}), "Math.max(a, b)", 0);
					}else{
						a = gtb(gl(s.major.start, s.major.prec, this.opt), {font: taFont}).w;
						b = gtb(gl(s.major.start + s.major.count * s.major.tick, s.major.prec, this.opt), {font: taFont}).w;
						c = gtb(gl(s.minor.start, s.minor.prec, this.opt), {font: taFont}).w;
						d = gtb(gl(s.minor.start + s.minor.count * s.minor.tick, s.minor.prec, this.opt), {font: taFont}).w;
						labelWidth = Math.max(a, b, c, d);
					}
					offsets.l = offsets.r = labelWidth / 2;
				}
			}
			return offsets;
		},
		render: function(dim, offsets){
			if(!this.dirty){ return this; }
			// prepare variable
			var start, stop, axisVector, tickVector, labelOffset, labelAlign,
				ta = this.chart.theme.axis,
				taStroke = "stroke" in this.opt ? this.opt.stroke : ta.stroke,
				taMajorTick = "majorTick" in this.opt ? this.opt.majorTick : ta.majorTick,
				taMinorTick = "minorTick" in this.opt ? this.opt.minorTick : ta.minorTick,
				taFont = "font" in this.opt ? this.opt.font : ta.font,
				taFontColor = "fontColor" in this.opt ? this.opt.fontColor : ta.fontColor,
				tickSize = Math.max(taMajorTick.length, taMinorTick.length),
				size = taFont ? g.normalizedLength(g.splitFontString(taFont).size) : 0;
			if(this.vertical){
				start = {y: dim.height - offsets.b};
				stop  = {y: offsets.t};
				axisVector = {x: 0, y: -1};
				if(this.opt.leftBottom){
					start.x = stop.x = offsets.l;
					tickVector = {x: -1, y: 0};
					labelAlign = "end";
				}else{
					start.x = stop.x = dim.width - offsets.r;
					tickVector = {x: 1, y: 0};
					labelAlign = "start";
				}
				labelOffset = {x: tickVector.x * (tickSize + labelGap), y: size * 0.4};
			}else{
				start = {x: offsets.l};
				stop  = {x: dim.width - offsets.r};
				axisVector = {x: 1, y: 0};
				labelAlign = "middle";
				if(this.opt.leftBottom){
					start.y = stop.y = dim.height - offsets.b;
					tickVector = {x: 0, y: 1};
					labelOffset = {y: tickSize + labelGap + size};
				}else{
					start.y = stop.y = offsets.t;
					tickVector = {x: 0, y: -1};
					labelOffset = {y: -tickSize - labelGap};
				}
				labelOffset.x = 0;
			}

			// render shapes

			this.cleanGroup();

			try{
				var s = this.group, c = this.scaler, t = this.ticks, canLabel,
					f = lin.getTransformerFromModel(this.scaler),
					forceHtmlLabels = dojox.gfx.renderer == "canvas",
					labelType = forceHtmlLabels || this.opt.htmlLabels && !dojo.isIE && !dojo.isOpera ? "html" : "gfx",
					dx = tickVector.x * taMajorTick.length,
					dy = tickVector.y * taMajorTick.length;

				s.createLine({x1: start.x, y1: start.y, x2: stop.x, y2: stop.y}).setStroke(taStroke);

				dojo.forEach(t.major, function(tick){
					var offset = f(tick.value), elem,
						x = start.x + axisVector.x * offset,
						y = start.y + axisVector.y * offset;
						s.createLine({
							x1: x, y1: y,
							x2: x + dx,
							y2: y + dy
						}).setStroke(taMajorTick);
						if(tick.label){
							elem = dc.axis2d.common.createText[labelType]
											(this.chart, s, x + labelOffset.x, y + labelOffset.y, labelAlign,
												tick.label, taFont, taFontColor);
							if(labelType == "html"){ this.htmlElements.push(elem); }
						}
				}, this);

				dx = tickVector.x * taMinorTick.length;
				dy = tickVector.y * taMinorTick.length;
				canLabel = c.minMinorStep <= c.minor.tick * c.bounds.scale;
				dojo.forEach(t.minor, function(tick){
					var offset = f(tick.value), elem,
						x = start.x + axisVector.x * offset,
						y = start.y + axisVector.y * offset;
						s.createLine({
							x1: x, y1: y,
							x2: x + dx,
							y2: y + dy
						}).setStroke(taMinorTick);
						if(canLabel && tick.label){
							elem = dc.axis2d.common.createText[labelType]
											(this.chart, s, x + labelOffset.x, y + labelOffset.y, labelAlign,
												tick.label, taFont, taFontColor);
							if(labelType == "html"){ this.htmlElements.push(elem); }
						}
				}, this);

				// use minor ticks for now
				//dx = tickVector.x * taMicroTick.length;
				//dy = tickVector.y * taMicroTick.length;
				dojo.forEach(t.micro, function(tick){
					var offset = f(tick.value), elem,
						x = start.x + axisVector.x * offset,
						y = start.y + axisVector.y * offset;
						s.createLine({
							x1: x, y1: y,
							x2: x + dx,
							y2: y + dy
						}).setStroke(taMinorTick);	// use minor tick for now
				}, this);
			}catch(e){
				// squelch
			}

			this.dirty = false;
			return this;
		}
	});
})();
