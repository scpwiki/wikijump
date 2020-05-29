dojo.provide("dojox.dtl.contrib.html");

dojo.require("dojox.dtl.html");

(function(){
	var dd = dojox.dtl;
	var ddch = dd.contrib.html;

	ddch.StyleNode = dojo.extend(function(styles){
		this.contents = {};
		this._styles = styles;
		for(var key in styles){
			this.contents[key] = new dd.Template(styles[key]);
		}
	},
	{
		render: function(context, buffer){
			for(var key in this.contents){
				dojo.style(buffer.getParent(), key, this.contents[key].render(context));
			}
			return buffer;
		},
		unrender: function(context, buffer){
			return buffer;
		},
		clone: function(buffer){
			return new this.constructor(this._styles);
		}
	});

	ddch.BufferNode = dojo.extend(function(nodelist, options){
		this.nodelist = nodelist;
		this.options = options;
	},
	{
		_swap: function(type, node){
			if(!this.swapped && this.parent.parentNode){
				if(type == "node"){
					if((node.nodeType == 3 && !this.options.text) || (node.nodeType == 1 && !this.options.node)){
						return;
					}
				}else if(type == "class"){
					if(type != "class"){
						return;
					}
				}

				this.onAddNode && dojo.disconnect(this.onAddNode);
				this.onRemoveNode && dojo.disconnect(this.onRemoveNode);
				this.onChangeAttribute && dojo.disconnect(this.onChangeAttribute);
				this.onChangeData && dojo.disconnect(this.onChangeData);

				this.swapped = this.parent.cloneNode(true);
				this.parent.parentNode.replaceChild(this.swapped, this.parent);
			}
		},
		render: function(context, buffer){
			this.parent = buffer.getParent();
			if(this.options.node){
				this.onAddNode = dojo.connect(buffer, "onAddNode", dojo.hitch(this, "_swap", "node"));
				this.onRemoveNode = dojo.connect(buffer, "onRemoveNode", dojo.hitch(this, "_swap", "node"));
			}
			if(this.options.text){
				this.onChangeData = dojo.connect(buffer, "onChangeData", dojo.hitch(this, "_swap", "node"));
			}
			if(this.options["class"]){
				this.onChangeAttribute = dojo.connect(buffer, "onChangeAttribute", dojo.hitch(this, "_swap", "class"));
			}

			buffer = this.nodelist.render(context, buffer);

			if(this.swapped){
				this.swapped.parentNode.replaceChild(this.parent, this.swapped);
				dojo._destroyElement(this.swapped);
			}else{
				this.onAddNode && dojo.disconnect(this.onAddNode);
				this.onRemoveNode && dojo.disconnect(this.onRemoveNode);
				this.onChangeAttribute && dojo.disconnect(this.onChangeAttribute);
				this.onChangeData && dojo.disconnect(this.onChangeData);
			}

			delete this.parent;
			delete this.swapped;
			return buffer;
		},
		unrender: function(context, buffer){
			return this.nodelist.unrender(context, buffer);
		},
		clone: function(buffer){
			return new this.constructor(this.nodelist.clone(buffer), this.options);
		}
	});

	dojo.mixin(ddch, {
		buffer: function(parser, token){
			// summary:
			//		Buffer large DOM manipulations during re-render.
			//	description:
			//		When using HtmlTemplate, wrap any content
			//		that you expect to change often during
			//		re-rendering. It will then remove its parent
			//		from the main document while it re-renders that
			//		section of code. It will only remove it from
			//		the main document if a mainpulation of somes sort
			//		happens. ie It won't swap out if it diesn't have to.
			// example:
			//		By default, it considers only node addition/removal
			//		to be "changing"
			//
			//		|	{% buffer %}{% for item in items %}<li>{{ item }}</li>{% endfor %}{% endbuffer %}
			// example:
			//		You can explicitly declare options:
			//
			//			* node: Watch node removal/addition
			//			* class: Watch for a classname to be changed
			//			* text: Watch for any text to be changed
			//
			//		|	{% buffer node class %}{% for item in items %}<li>{{ item }}</li>{% endfor %}{% endbuffer %}
			var parts = token.contents.split().slice(1);
			var options = {};
			var found = false;
			for(var i=parts.length; i--;){
				found = true;
				options[parts[i]] = true;
			}
			if(!found){
				options.node = true;
			}
			var nodelist = parser.parse(["endbuffer"]);
			parser.next_token();
			return new ddch.BufferNode(nodelist, options);
		},
		html: function(parser, token){
			dojo.deprecated("{% html someVariable %}", "Use {{ someVariable|safe }} instead")
			return parser.create_variable_node(token.contents.slice(5) + "|safe");
		},
		tstyle: function(parser, token){
			var styles = {};
			token = token.contents.replace(/^tstyle\s+/, "");
			var rules = token.split(/\s*;\s*/g);
			for(var i = 0, rule; rule = rules[i]; i++){
				var parts = rule.split(/\s*:\s*/g);
				var key = parts[0];
				var value = parts[1];
				if(value.indexOf("{{") == 0){
					styles[key] = value;
				}
			}
			return new ddch.StyleNode(styles);
		}
	});

	dd.register.tags("dojox.dtl.contrib", {
		"html": ["html", "attr:tstyle", "buffer"]
	});
})();