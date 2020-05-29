dojo.provide("dojox.gfx.utils");

dojo.require("dojox.gfx");

(function(){
	var d = dojo, g = dojox.gfx, gu = g.utils;

	dojo.mixin(gu, {
		forEach: function(
			/* dojox.gfx.Surface || dojox.gfx.Shape */ object,
			/*Function|String|Array*/ f, /*Object?*/ o
		){
			o = o || d.global;
			f.call(o, object);
			if(object instanceof g.Surface || object instanceof g.Group){
				d.forEach(object.children, function(shape){
					gu.inspect(shape, f, o);
				});
			}
		},

		serialize: function(
			/* dojox.gfx.Surface || dojox.gfx.Shape */ object
		){
			var t = {}, v, isSurface = object instanceof g.Surface;
			if(isSurface || object instanceof g.Group){
				t.children = d.map(object.children, gu.serialize);
				if(isSurface){
					return t.children;	// Array
				}
			}else{
				t.shape = object.getShape();
			}
			if(object.getTransform){
				v = object.getTransform();
				if(v){ t.transform = v; }
			}
			if(object.getStroke){
				v = object.getStroke();
				if(v){ t.stroke = v; }
			}
			if(object.getFill){
				v = object.getFill();
				if(v){ t.fill = v; }
			}
			if(object.getFont){
				v = object.getFont();
				if(v){ t.font = v; }
			}
			return t;	// Object
		},

		toJson: function(
			/* dojox.gfx.Surface || dojox.gfx.Shape */ object,
			/* Boolean? */ prettyPrint
		){
			return d.toJson(gu.serialize(object), prettyPrint);	// String
		},

		deserialize: function(
			/* dojox.gfx.Surface || dojox.gfx.Shape */ parent,
			/* dojox.gfx.Shape || Array */ object
		){
			if(object instanceof Array){
				return d.map(object, d.hitch(null, gu.serialize, parent));	// Array
			}
			var shape = ("shape" in object) ? parent.createShape(object.shape) : parent.createGroup();
			if("transform" in object){
				shape.setTransform(object.transform);
			}
			if("stroke" in object){
				shape.setStroke(object.stroke);
			}
			if("fill" in object){
				shape.setFill(object.fill);
			}
			if("font" in object){
				shape.setFont(object.font);
			}
			if("children" in object){
				d.forEach(object.children, d.hitch(null, gu.deserialize, shape));
			}
			return shape;	// dojox.gfx.Shape
		},

		fromJson: function(
			/* dojox.gfx.Surface || dojox.gfx.Shape */ parent,
			/* String */ json
		){
			return gu.deserialize(parent, d.fromJson(json));	// Array || dojox.gfx.Shape
		}
	});
})();
