dojo.provide("demos.fonts.src.pie");

dojo.require("dojox.gfx");
dojo.require("dojox.gfx.VectorText");
dojo.require("dojox.gfx.utils");

(function(){
	demos.fonts.src.pie = function(node, font, data, args){
		args = args || {};
		this.font = font;
		this.data = data || [];
		this.stroke = { color: args.stroke || "white", width: 1 };
		this.fill = args.fill || "#40b337";
		this.radius = args.radius || 112;
		this.center = args.center || 114;
		this.width = args.width || 228;
		this.height = args.height || 228;
		this.domNode = node || null;
		this.rotation = args.rotation || 0;
		this.mainFill = args.main || "#364c42";
		this.surface = null;
		this.otherData = [];
	};

	dojo.extend(demos.fonts.src.pie, {
		findScale: function(txt, w){
			//	we do this manually here because there's some issues with font fitting in the font code.
			//	so it's a hack for now.
			var g = dojo.map(this.font._normalize(txt).split(""), function(chr){
				return this.font.glyphs[chr] || { path:"", xAdvance: this.font.advance.missing.x };
			}, this);
			
			var cw = 0;
			for(var i=0; i<g.length; i++){
				var tw = g[i].xAdvance;
				if(i+1<g.length && g[i].kern && g[i+1] && g[i+1].code && g[i].kern[g[i+1].code]){
					tw += g[i].kern[g[i+1].code].x;
				}
				cw += tw;
			}
			return this.font._round(w/cw);
		},
		draw: function(d){
			var m=dojox.gfx.matrix, rad=(2*Math.PI)/360;
			var r=this.radius, c=this.center, data=(this.data=d||this.data);

			if(!this.surface && this.domNode){
				this.surface = dojox.gfx.createSurface(this.domNode, this.width, this.height);
			}
			this.surface.clear();

			//	prepare our data.
			var sum=0, idx=-1, max=0, valueLimit=5, other, otherSum, pct=0;
			dojo.forEach(data, function(item){ sum += item.value; });
			dojo.forEach(data, function(item, i){
				item.pct = Math.round(item.value * (100/sum));
				pct += item.pct;
				if(item.pct > max){
					max = item.pct;
					idx = i;
				}
				if(item.pct < valueLimit){
					if(!other){ other = { label: "Other", pct: 0, value: 0, isOther: true }; }
					other.pct += item.pct;
					other.value += item.value;
					this.otherData.push(item);
				}
			}, this);
			if(pct<100){ data[idx].pct += 100 - pct; }
			if(other){
				data.push(other); 
			}

			//	create the main group.
			var g = this.surface.createGroup(), mm = m.rotategAt(this.rotation, c, c);

			//	rotate the whole chart
			g.setTransform(mm);

			//	the background circle.
			g.createCircle({ r: r, cx: c, cy: c })
				.setStroke(this.stroke)
				.setFill(this.mainFill);

			//	figure out the start and end angles for the arc overlay
			var startAngle = 0,
				endAngle = (360*(100-data[idx].pct))/100,
				offset = (360-endAngle)/2; 

			//	now that we know where the start angle is really supposed to be, recalculate start and end.
			startAngle = offset, endAngle = (startAngle+(360*(100-data[idx].pct))/100)%360;

			//	do the biggest slice
			var info = g.createGroup(), 
				txtArgs = { text: data[idx].label.toUpperCase(), x:0, y:0, width:r/1.65, height:r/1.65, align:"start", fitting:dojox.gfx.vectorFontFitting.NONE },
				fontArgs = { size: "10px" };
			var txt = this.font.draw(info, txtArgs, fontArgs, this.stroke.color), 
				txtscale = this.findScale(data[idx].label.toUpperCase(), r/1.65), 
				txty = Math.round(this.font.viewbox.height*txtscale)/2;
			if(this.rotation>90 && this.rotation<270){
				txt.setTransform(new dojox.gfx.Matrix2D([
					m.translate((r*2)-10, c+txty),
					m.scale(txtscale),
					m.rotateg(180)
				]));
			} else {
				txt.setTransform(new dojox.gfx.Matrix2D([
					m.translate((r*2)-6-(r/1.65), c-txty),
					m.scale(txtscale)
				]));
			}

			var pct = this.font.draw(
				info,
				{ text: (data[idx].pct).toString(), width: 24, height: 24, align:"start", fitting:dojox.gfx.vectorFontFitting.FLOW, x:0, y:0 },
				fontArgs,
				this.stroke.color
			);
			if(this.rotation>90 && this.rotation<270){
				pct.setTransform(new dojox.gfx.Matrix2D([
					m.translate(r+18, c+5),
					pct.getTransform(),
					m.rotateg(180)
				]));
			} else {
				pct.setTransform(new dojox.gfx.Matrix2D([
					m.translate(r+12, c-5),
					pct.getTransform()
				]));
			}

			//	draw the background for the rest of the data
			var main = g.createGroup(), st=Math.min(startAngle, endAngle), ed=Math.max(startAngle, endAngle);

			var arc = main.createPath({})
				.moveTo(c, c)
				.lineTo(c+r*Math.cos(st*rad), c+r*Math.sin(st*rad))
				.arcTo(r, r, 0, (Math.max(startAngle, endAngle)-Math.min(startAngle, endAngle)>180), true, { x:c+r*Math.cos(ed*rad), y:c+r*Math.sin(ed*rad) })
				.lineTo(c, c)
				.closePath()
				.setStroke(this.stroke)
				.setFill(this.fill);

			//	create the rest of the pie slice data
			var lastAngle=startAngle;
			for(var i=idx+1; (i%data.length)!=idx; i++){
				var d=data[i%data.length], v=d.pct, a=lastAngle;
				if(d.pct<valueLimit && !d.isOther){ continue; }

				lastAngle += (d.pct*360)/100;
				if(i%data.length!=idx-1){
					main.createLine({ x1:c, y1: c, x2: c+r-8, y2: c})
						.setStroke(this.stroke)
						.setTransform(m.rotategAt(lastAngle, c, c));
				}

				//	lets calculate some dimensions here for font fitting.
				var _0 = (a+1)*rad;
				var _1 = (lastAngle-1)*rad;
				var _center = lastAngle-((lastAngle-a)/2);
				var _2 = _center*rad;

				//	if this is the "other", define our anchor points.
				if(d.isOther){
					//	we add the rotation because this is a virtual point, not actually plotted.
					this.otherPoint = {
						x: Math.round(c+(38*Math.cos(_2+(this.rotation*rad)))),
						y: Math.round(c+(38*Math.sin(_2+(this.rotation*rad))))
					};

					//	we already know where the center is, these would be the end points of the Other arc.
					this.otherEnds = {
						small: { x: c+r*Math.cos(_0), y: c+r*Math.sin(_0) },
						large: { x: c+r*Math.cos(_1), y: c+r*Math.sin(_1) }
					};

					//	determine the direction.
					var quadrant = lastAngle + this.rotation;
					if(quadrant <= 90){ this.otherDirection="SE"; }
					else if(quadrant > 90 && quadrant <= 180){ this.otherDirection="SW"; }
					else if(quadrant > 180 && quadrant <= 270){ this.otherDirection="NW"; }
					else { this.otherDirection="NE"; }
				}

				var spoke = {
					x1: c+((r*0.36)*Math.cos(_2)), y1: c+((r*0.36)*Math.sin(_2)),
					x2: c+((r-16)*Math.cos(_2)), y2: c+((r-16)*Math.sin(_2))
				};
				spoke.dx = spoke.x2 - spoke.x1, spoke.dy = spoke.y2 - spoke.y1;

				var tiny={
					x1: c+(32*Math.cos(_0)), x2: c+(32*Math.cos(_1)),
					y1: c+(32*Math.sin(_0)), y2: c+(32*Math.sin(_1))
				};
				tiny.dx = tiny.x2 - tiny.x1, tiny.dy = tiny.y2 - tiny.y1;

				var small={
					x1: c+((r*0.4)*Math.cos(_0)), x2: c+((r*0.4)*Math.cos(_1)),
					y1: c+((r*0.4)*Math.sin(_0)), y2: c+((r*0.4)*Math.sin(_1))
				};
				small.dx = small.x2 - small.x1, small.dy = small.y2 - small.y1;

				//	whats the height limit?  the inner distance.
				var maxw = Math.sqrt(spoke.dx*spoke.dx + spoke.dy*spoke.dy);
				var maxh = Math.sqrt(small.dx*small.dx + small.dy*small.dy);
				var tinyh = Math.sqrt(tiny.dx*tiny.dx + tiny.dy*tiny.dy);

				//	the value
				var label = main.createGroup(), 
					txtArgs = { text: dojo.trim(d.label.toUpperCase()), x:0, y:0, width:r, height:r, align:"start", fitting:dojox.gfx.vectorFontFitting.NONE },
					fontArgs = { size: "8px" };
				label.setTransform(m.rotategAt(lastAngle-((lastAngle-a)/2), c, c));

				var pct = this.font.draw(
					label,
					{ text: (d.pct).toString(), width: 12, height: 12, align:"start", fitting:dojox.gfx.vectorFontFitting.FLOW, x:0, y:0 },
					fontArgs,
					this.stroke.color
				);

				//	figure out if we need to scale it down.
				var pctscale=0;
				if(tinyh < 8){
					//	14 being the pixel scale we rendered the font at.
					pctscale = tinyh/this.font.viewbox.height;
				}
				if(_center>=90-this.rotation && _center<=270-this.rotation){
					mtrx=new dojox.gfx.Matrix2D([
						m.translate(r+24, c+(pctscale?Math.round(this.font.viewbox.height*pctscale)/2:3)), 
						pctscale?m.scale(pctscale):pct.getTransform(),
						m.rotateg(180)
					]);
				} else {
					mtrx=new dojox.gfx.Matrix2D([
						m.translate(r+20, c-(pctscale?Math.round(this.font.viewbox.height*pctscale)/2:3)),
						pctscale?m.scale(pctscale):pct.getTransform()
					]);
				}
				pct.setTransform(mtrx);

				var txt = this.font.draw(label, txtArgs, fontArgs, this.stroke.color), 
					txtscale = this.findScale(d.label.toUpperCase(), r/1.65), 
					txty = Math.round(this.font.viewbox.height*txtscale)/2,
					txtw = this.font.getWidth(d.label.toUpperCase(), txtscale);
				if(Math.round(this.font.viewbox.height*txtscale) > maxh){
					//	recalc the scale based on height
					txtscale = maxh/this.font.viewbox.height;
					txty = Math.round(this.font.viewbox.height*txtscale)/2;
					txtw = this.font.getWidth(d.label.toUpperCase(), txtscale);
				}

				if(_center>=90-this.rotation && _center<=270-this.rotation){
					var mtrx=new dojox.gfx.Matrix2D([
						m.translate(r*2-10, c+txty), 
						m.scale(txtscale),
						m.rotateg(180)
					]);
				} else {
					var mtrx=new dojox.gfx.Matrix2D([
						m.translate((r*2)-txtw-6, c-txty), 
						m.scale(txtscale)
					]);
				}
				txt.setTransform(mtrx);
			}
			return g; //	dojox.gfx.Group
		}
	});
})();
