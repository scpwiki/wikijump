dojo.provide("dojox.widget.ColorPicker");
dojo.experimental("dojox.widget.ColorPicker"); // level: beta

dojo.require("dijit.form._FormWidget");
dojo.require("dojo.dnd.move"); 
dojo.require("dojo.fx"); 

(function(){

	// this ported directly from 0.4 dojo.gfx.colors.hsv, with bugs :)
	// FIXME: use ttrenka's HSB ?
	var _hsv2rgb = function(/* int || Array */h, /* int */s, /* int */v, /* Object? */options){
		//	summary
		//	converts an HSV value set to RGB, ranges depending on optional options object.
		//	patch for options by Matthew Eernisse 	
		if (dojo.isArray(h)) {
			if(s){
				options = s;
			}
			v = h[2] || 0;
			s = h[1] || 0;
			h = h[0] || 0;
		}

		var opt = {
			inputRange:  (options && options.inputRange)  ? options.inputRange : [255, 255, 255],
			outputRange: (options && options.outputRange) ? options.outputRange : 255
		};

	    switch(opt.inputRange[0]) { 
			// 0.0-1.0 
			case 1: h = h * 360; break; 
			// 0-100 
			case 100: h = (h / 100) * 360; break; 
			// 0-360 
			case 360: h = h; break; 
			// 0-255 
			default: h = (h / 255) * 360; 
		} 
		if (h == 360){ h = 0;}

		//	no need to alter if inputRange[1] = 1
		switch(opt.inputRange[1]){
			case 100: s /= 100; break;
			case 255: s /= 255;
		}

		//	no need to alter if inputRange[1] = 1
		switch(opt.inputRange[2]){
			case 100: v /= 100; break;
			case 255: v /= 255;
		}

		var r = null;
		var g = null;
		var b = null;

		if (s == 0){
			// color is on black-and-white center line
			// achromatic: shades of gray
			r = v;
			g = v;
			b = v;
		}else{
			// chromatic color
			var hTemp = h / 60;		// h is now IN [0,6]
			var i = Math.floor(hTemp);	// largest integer <= h
			var f = hTemp - i;		// fractional part of h

			var p = v * (1 - s);
			var q = v * (1 - (s * f));
			var t = v * (1 - (s * (1 - f)));

			switch(i){
				case 0: r = v; g = t; b = p; break;
				case 1: r = q; g = v; b = p; break;
				case 2: r = p; g = v; b = t; break;
				case 3: r = p; g = q; b = v; break;
				case 4: r = t; g = p; b = v; break;
				case 5: r = v; g = p; b = q; break;
			}
		}

		switch(opt.outputRange){
			case 1:
				r = dojo.math.round(r, 2);
				g = dojo.math.round(g, 2);
				b = dojo.math.round(b, 2);
				break;
			case 100:
				r = Math.round(r * 100);
				g = Math.round(g * 100);
				b = Math.round(b * 100);
				break;
			default:
				r = Math.round(r * 255);
				g = Math.round(g * 255);
				b = Math.round(b * 255);
		}
		return [r, g, b];

	};
	
	var webSafeFromHex = function(hex){
		// stub, this is planned later:
		return hex;
	}
		
	dojo.declare("dojox.widget.ColorPicker",
		dijit.form._FormWidget,
		{
		// summary: a HSV color picker - similar to Photoshop picker
		//
		// description: 
		//		Provides an interactive HSV ColorPicker similar to
		//		PhotoShop's color selction tool. This is an enhanced 
		//		version of the default dijit.ColorPalette, though provides
		//		no accessibility.
		// example:
		// |	var picker = new dojox.widget.ColorPicker({
		// |		// a couple of example toggles:
		// |		animatePoint:false,
		// |		showHsv: false,
		// |		webSafe: false,
		// |		showRgb: false 	
		// |	});
		//	
		// example: 
		// | 	<!-- markup: -->
		// | 	<div dojoType="dojox.widget.ColorPicker"></div>
		//
		//
		// showRgb: Boolean
		//	show/update RGB input nodes
		showRgb: true,
	
		// showHsv: Boolean
		//	show/update HSV input nodes
		showHsv: true,
	
		// showHex: Boolean
		//	show/update Hex value field
		showHex: true,

		// webSafe: Boolean
		//	deprecated? or just use a toggle to show/hide that node, too?
		webSafe: true,

		// animatePoint: Boolean
		//	toggle to use slideTo (true) or just place the cursor (false) on click
		animatePoint: true,

		// slideDuration: Integer
		//	time in ms picker node will slide to next location (non-dragging) when animatePoint=true
		slideDuration: 250, 

		// liveUpdate: Boolean
		//		Set to true to fire onChange in an indeterminate way
		liveUpdate: false, 

		_underlay: dojo.moduleUrl("dojox.widget","ColorPicker/images/underlay.png"),
		templatePath: dojo.moduleUrl("dojox.widget","ColorPicker/ColorPicker.html"),

		postCreate: function(){
			// summary: As quickly as we can, set up ie6 alpha-filter support for our
			// 	underlay.  we don't do image handles (done in css), just the 'core' 
			//	of this widget: the underlay. 
			if(dojo.isIE<7){ 
				this.colorUnderlay.style.filter = "progid:DXImageTransform.Microsoft.AlphaImageLoader(src='"+this._underlay+"', sizingMethod='scale')";
				this.colorUnderlay.src = this._blankGif.toString();
			}
			// hide toggle-able nodes:
			if(!this.showRgb){ this.rgbNode.style.display = "none"; }
			if(!this.showHsv){ this.hsvNode.style.display = "none"; }
			if(!this.showHex){ this.hexNode.style.display = "none"; } 
			if(!this.webSafe){ this.safePreviewNode.style.visibility = "hidden"; } 
		},

		startup: function(){

			// this._offset = ((dojo.marginBox(this.cursorNode).w)/2); 
			this._offset = 0; 
			var cmb = dojo.marginBox(this.cursorNode);
			var hmb = dojo.marginBox(this.hueCursorNode);
			this._shift = {
				hue: {
					x: Math.round(hmb.w/2)-1,
					y: Math.round(hmb.h/2)-1
				},
				picker: {
					x: Math.floor(cmb.w / 2),
					y: Math.floor(cmb.h / 2)
				}
			};

			var ox = this._shift.picker.x;
			var oy = this._shift.picker.y;
			this._mover = new dojo.dnd.Moveable(this.cursorNode, {
				mover: dojo.dnd.boxConstrainedMover({ 
					t:0 - oy, 
					l:0 - ox, 
					w:151, h:151
				})
			}); 
			
			this._hueMover = new dojo.dnd.Moveable(this.hueCursorNode, {
				mover: dojo.dnd.boxConstrainedMover({ 
					t:0 - this._shift.hue.y, 
					l:0, w:0, h:150 
				})
			});

			// no dnd/move/move published ... use a timer:
			dojo.subscribe("/dnd/move/stop", dojo.hitch(this, "_clearTimer"));
			dojo.subscribe("/dnd/move/start", dojo.hitch(this, "_setTimer"));

			// ugly scaling calculator.  need a XYslider badly
			this._sc = (1 / dojo.coords(this.colorUnderlay).w);  
			this._hueSc = 255/150;
			
			// initial color
			this._updateColor(); 
		
		},

		_setTimer: function(/* dojo.dnd.Mover */mover){
			// FIXME: should I assume this? focus on mouse down so on mouse up
			dijit.focus(mover.node);
			dojo.setSelectable(this.domNode,false);
			this._timer = setInterval(dojo.hitch(this, "_updateColor"), 45);	
		},
	
		_clearTimer: function(/* dojo.dnd.Mover */mover){
			clearInterval(this._timer);
			this._timer = null;
			this.onChange(this.value);
			dojo.setSelectable(this.domNode,true);
		},

		_setHue: function(/* Decimal */h){
			// summary: sets a natural color background for the 
			// 	underlay image against closest hue value (full saturation) 
			// h: 0..255 

			// this is not a pretty conversion:
			var hue = dojo.colorFromArray(_hsv2rgb(h, 1, 1, { inputRange: 1 })).toHex();
			dojo.style(this.colorUnderlay, "backgroundColor", hue);

		},

		_updateColor: function(){
			// summary: update the previewNode color, and input values [optional]
			
			// all these should be in the [0..150] range
			var _huetop = dojo.style(this.hueCursorNode,"top") + this._shift.hue.y; 
			var _pickertop = dojo.style(this.cursorNode,"top") + this._shift.picker.y;
			var _pickerleft = dojo.style(this.cursorNode,"left") + this._shift.picker.x;
			
			var h = Math.round(255 - (_huetop * this._hueSc));
			var s = Math.round(_pickerleft * this._sc * 100); 
			var v = Math.round(100 - (_pickertop * this._sc ) *100);

			// limit hue calculations to only when it changes
			if(h != this._hue){ this._setHue(h); }

			var rgb = _hsv2rgb(h, s/100, v/100,{ inputRange: 1 }); 
			var hex = dojo.colorFromArray(rgb).toHex();

			this.previewNode.style.backgroundColor = hex;	
			if(this.webSafe){ this.safePreviewNode.style.backgroundColor = webSafeFromHex(hex); }
			if(this.showHex){ this.hexCode.value = hex; }
			if(this.showRgb){
				this.Rval.value = rgb[0];
				this.Gval.value = rgb[1];	
				this.Bval.value = rgb[2];
			}
			if(this.showHsv){
				this.Hval.value = Math.round((h * 360) / 255); // convert to 0..360
				this.Sval.value = s;
				this.Vval.value = v;
			}
			this.value = hex;

			// anytime we muck with the color, fire onChange?
			if ((!this._timer && !arguments[1]) || this.liveUpdate){
				this.setAttribute("value", hex);	
				this.onChange(hex);
			}
		},

		_updatePoints: function(e){
//			update all other values and move points on picker to reflect next values
//			entered in the HEX:
//			console.log('changed? set points to:', e.target.value);
			// not looking forward to the hex->hsv->pixel coordinate reverse calculation
		},

		_setHuePoint: function(/* Event */evt){ 
			// summary: set the hue picker handle on relative y coordinates
			var ypos = evt.layerY - this._shift.hue.y;
			if(this.animatePoint){
				dojo.fx.slideTo({ 
					node: this.hueCursorNode, 
					duration:this.slideDuration,
					top: ypos,
					left: 0,
					onEnd: dojo.hitch(this, "_updateColor", true)
				}).play();
			}else{
				dojo.style(this.hueCursorNode, "top", ypos + "px");
				this._updateColor(false); 
			}
		},

		_setPoint: function(/* Event */evt){
			// summary: set our picker point based on relative x/y coordinates
		//	evt.preventDefault();
			var newTop = evt.layerY - this._shift.picker.y;
			var newLeft = evt.layerX - this._shift.picker.x;
			if(evt){ dijit.focus(evt.target); }

			if(this.animatePoint){
				dojo.fx.slideTo({ 
					node: this.cursorNode, 
					duration:this.slideDuration,
					top: newTop,
					left: newLeft,
					onEnd: dojo.hitch(this,"_updateColor", true)
				}).play();
			}else{
				dojo.style(this.cursorNode, {
					left: newLeft + "px",
					top: newTop + "px"	
				});
				this._updateColor(false); 
			}
		},
		
		_handleKey: function(e){
			// FIXME: not implemented YET
			var keys = dojo.keys;
		}
		
	});

})();
