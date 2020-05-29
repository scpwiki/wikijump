dojo.provide("dojox.dtl.contrib.data");
dojo.require("dojox.dtl._base");

(function(){
	var dd = dojox.dtl;
	var ddcd = dd.contrib.data;

	var first = true;

	ddcd._BoundItem = dojo.extend(function(item, store){
		this.item = item;
		this.store = store;
	},
	{
		get: function(key){
			var store = this.store;
			var item = this.item;

			if(key == "getLabel"){
				return store.getLabel(item);
			}else if(key == "getAttributes"){
				return store.getAttributes(item);
			}else if(key == "getIdentity"){
				if(store.getIdentity){
					return store.getIdentity(item);
				}
				return "Store has no identity API";
			}else{
				if(!store.hasAttribute(item, key)){
					if(key.slice(-1) == "s"){
						if(first){
							first = false;
							dojo.deprecated("You no longer need an extra s to call getValues, it can be figured out automatically");
						}
						key = key.slice(0, -1);
					}
					if(!store.hasAttribute(item, key)){
						return;
					}
				}

				var values = store.getValues(item, key);
				if(!values){
					return;
				}
				if(!dojo.isArray(values)){
					return new ddcd._BoundItem(values, store);
				}

				values = dojo.map(values, function(value){
					if(dojo.isObject(value) && store.isItem(value)){
						return new ddcd._BoundItem(value, store);
					}
					return value;
				});
				values.get = ddcd._get;
				return values;
			}
		}
	});

	ddcd.BindDataNode = dojo.extend(function(items, store, alias){
		this.items = new dd._Filter(items);
		this.store = new dd._Filter(store);
		this.alias = alias;
	},
	{
		render: function(context, buffer){
			var items = this.items.resolve(context);
			var store = this.store.resolve(context);
			if(!store){
				throw new Error("data_bind didn't receive a store");
			}

			var list = [];
			if(items){
				for(var i = 0, item; item = items[i]; i++){
					list.push(new ddcd._BoundItem(item, store));
				}
			}

			context[this.alias] = list;
			return buffer;
		},
		unrender: function(context, buffer){
			return buffer;
		},
		clone: function(){
			return this;
		}
	});

	dojo.mixin(ddcd, {
		_get: function(key){
			if(this.length){
				return this[0].get(key);
			}
		},
		bind_data: function(parser, token){
			var parts = token.contents.split();

			if(parts[2] != 'to' || parts[4] != 'as' || !parts[5]){
				throw new Error("data_bind expects the format: 'data_bind items to store as varName'");
			}

			return new ddcd.BindDataNode(parts[1], parts[3], parts[5]);
		}
	});

	dd.register.tags("dojox.dtl.contrib", {
		"data": ["bind_data"]
	});
})();