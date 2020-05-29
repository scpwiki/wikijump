dojo.provide("dojox.fx.flip");
dojo.experimental("dojox.fx.flip");
dojo.require("dojo.fx");
(function(){
	
	// because ShrinkSafe will eat this up: 
	var borderConst = "border",
		widthConst = "Width",
		heightConst = "Height",
		topConst = "Top",
		rightConst = "Right",
		leftConst = "Left",
		bottomConst = "Bottom"
	;

	dojox.fx.flip = function(/*Object*/ args){
		// summary: Animate a node flipping following a specific direction
		//	
		// description:
		//		Returns an animation that will flip the
		//		node around a central axis:
		//		if args.dir is "left" or "right" --> y axis
		//		if args.dir is "top" or "bottom" --> x axis
		//
		//		This effect is obtained using a border distorsion applied to a helper node.
		//
		//		The user can specify three background colors for the helper node:
		//		darkColor: the darkest color reached during the animation 
		//		lightColor: the brightest color
		//		endColor: the final backgroundColor for the node
		//
		//	example:
		//	|	var anim = dojox.fx.flip({ 
		//	|		node: dojo.byId("nodeId"),
		//	|		dir: "top",
		//	|		darkColor: "#555555",
		//	|		lightColor: "#dddddd",
		//	|		endColor: "#666666",
		//	|		duration:300
		//	|	  });

		var helperNode = dojo.doc.createElement("div");

		var node = args.node = dojo.byId(args.node), 
			s = node.style,
			dims = null, 
			hs = null, 
			pn = null,
			lightColor = args.lightColor || "#dddddd", 
			darkColor = args.darkColor || "#555555",
			bgColor = dojo.style(node, "backgroundColor"), 
			endColor = args.endColor || bgColor,
			staticProps = {}, 
			anims = [],
			duration = args.duration ? args.duration / 2 : 250,
			dir = args.dir || "left", 
			pConst = 0.6, 
			transparentColor = "transparent",
			whichAnim = args.whichAnim,
			mConst = 1
		;

		// IE6 workaround: IE6 doesn't support transparent borders
		var convertColor = function(color){
			return ((new dojo.Color(color)).toHex() === "#000000") ? "#000001" : color;
		};

		if(dojo.isIE < 7){
			endColor = convertColor(endColor);
			lightColor = convertColor(lightColor);
			darkColor = convertColor(darkColor);
			bgColor = convertColor(bgColor);
			transparentColor = "black";
			helperNode.style.filter = "chroma(color='#000000')";
		}

		var init = (function(n){
			return function(){
				var ret = dojo.coords(n, true);
				dims = {
					top: ret.y,
					left: ret.x,
					width: ret.w,
					height: ret.h
				};
			}
		})(node);
		init();
		if(whichAnim){
			pConst = .5;//.6875;
			mConst = 0;
		}
		// helperNode initialization
		hs = {
			position: "absolute",
			top: dims["top"] + "px",
			left: dims["left"] + "px",
			height: "0",
			width: "0",
			zIndex: args.zIndex || (s.zIndex || 0),
			border: "0 solid " + transparentColor,
			fontSize: "0",
			visibility: "hidden"
		};
		dims["endHeight"] = dims["height"] * pConst; 
		dims["endWidth"] = dims["width"] * pConst; 
		var props = [ {}, 
			{
				top: dims["top"],
				left: dims["left"]
			}
		];
		var dynProperties = {
			left: [leftConst, rightConst, topConst, bottomConst, widthConst, heightConst, "end" + heightConst, leftConst],
			right: [rightConst, leftConst, topConst, bottomConst, widthConst, heightConst, "end" + heightConst, leftConst],
			top: [topConst, bottomConst, leftConst, rightConst, heightConst, widthConst, "end" + widthConst, topConst],
			bottom: [bottomConst, topConst, leftConst, rightConst, heightConst, widthConst, "end" + widthConst, topConst] 
		};
		// property names
		pn = dynProperties[dir];

		staticProps[pn[5].toLowerCase()] = dims[pn[5].toLowerCase()] + "px";
		staticProps[pn[4].toLowerCase()] = "0";
		staticProps[borderConst + pn[1] + widthConst] = dims[pn[4].toLowerCase()] + "px";
		staticProps[borderConst + pn[1] + "Color"] = bgColor;

		var p0 = props[0];
		p0[borderConst + pn[1] + widthConst] = 0; 
		p0[borderConst + pn[1] + "Color"] = darkColor; 
		p0[borderConst + pn[2] + widthConst] = dims[pn[6]] / 2;
		p0[borderConst + pn[3] + widthConst] = dims[pn[6]] / 2;
		p0[pn[2].toLowerCase()] = dims[pn[2].toLowerCase()] - mConst * (dims[pn[6]] / 8);
		p0[pn[7].toLowerCase()] = dims[pn[7].toLowerCase()] + dims[pn[4].toLowerCase()] / 2 + (args.shift || 0);
		p0[pn[5].toLowerCase()] = dims[pn[6]];

		var p1 = props[1];
		p1[borderConst + pn[0] + "Color"] = { start: lightColor, end: endColor };
		p1[borderConst + pn[0] + widthConst] = dims[pn[4].toLowerCase()];
		p1[borderConst + pn[2] + widthConst] = 0;
		p1[borderConst + pn[3] + widthConst] = 0;
		p1[pn[5].toLowerCase()] = { start: dims[pn[6]], end: dims[pn[5].toLowerCase()] };

		dojo.mixin(hs, staticProps);
		dojo.style(helperNode, hs);
		dojo.body().appendChild(helperNode);

		var finalize = function(){
			helperNode.parentNode.removeChild(helperNode);
			// fixes a flicker when the animation ends
			s.backgroundColor = endColor;
			s.visibility = "visible";
		};
		if(whichAnim == "last"){
			for(var i in p0){
				p0[i] = { start: p0[i] };
			}
			p0[borderConst + pn[1] + "Color"] = { start: darkColor, end: endColor }; 
			p1 = p0;
		}
		if(!whichAnim || whichAnim == "first"){
			anims.push(dojo.animateProperty({
				node: helperNode, 
				duration: duration,
				properties: p0
			}));
		}
		if(!whichAnim || whichAnim == "last"){
			anims.push(dojo.animateProperty({
				node: helperNode, 
				duration: duration,
				properties: p1,
				onEnd: finalize
			}));
		}

		// hide the original node
		dojo.connect(anims[0], "play", function(){
			helperNode.style.visibility = "visible";
			s.visibility = "hidden"; 
		});

		return dojo.fx.chain(anims); // dojo._Animation

	}

	dojox.fx.flipCube = function(/*Object*/ args){
		// summary: An extension to `dojox.fx.flip` providing a more 3d-like rotation
		//
		// description:
		//		An extension to `dojox.fx.flip` providing a more 3d-like rotation. 
		//		Behaves the same as `dojox.fx.flip`, using the same attributes and 
		//		other standard `dojo._Animation` properties.
		//
		//	example:
		//		See `dojox.fx.flip`
		var anims = [],
			mb = dojo.marginBox(args.node),
			shiftX = mb.w / 2,
			shiftY = mb.h / 2,
			dims = {
				top: {
					pName: "height",
					args:[
						{
							whichAnim: "first",
							dir: "top",
							shift: -shiftY
						},
						{
							whichAnim: "last",
							dir: "bottom",
							shift: shiftY
						}
					]
				},
				right: {
					pName: "width",
					args:[
						{
							whichAnim: "first",
							dir: "right",
							shift: shiftX
						},
						{
							whichAnim: "last",
							dir: "left",
							shift: -shiftX
						}
					]
				},
				bottom: {
					pName: "height",
					args:[
						{
							whichAnim: "first",
							dir: "bottom",
							shift: shiftY
						},
						{
							whichAnim: "last",
							dir: "top",
							shift: -shiftY
						}
					]
				},
				left: {
					pName: "width",
					args:[
						{
							whichAnim: "first",
							dir: "left",
							shift: -shiftX
						},
						{
							whichAnim: "last",
							dir: "right",
							shift: shiftX
						}
					]
				}
			}
		;
		var d = dims[args.dir || "left"],
			p = d.args
		;
		
		for(var i = p.length - 1; i >= 0; i--){
			dojo.mixin(args, p[i]);
			anims.push(dojox.fx.flip(args));
		}
		return dojo.fx.combine(anims);
	};

		
})();
