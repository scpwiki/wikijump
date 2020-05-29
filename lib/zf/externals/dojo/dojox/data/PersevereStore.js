dojo.provide("dojox.data.PersevereStore");
dojo.require("dojox.data.JsonRestStore");
dojo.require("dojox.rpc.Client"); // Persevere supports this and it improves reliability
if(dojox.rpc.OfflineRest){
	dojo.require("dojox.json.query"); // this is so we can perform queries locally 
}

// PersevereStore is an extension of JsonRestStore to handle Persevere's special features

dojox.json.ref._useRefs = true; // Persevere supports referencing
dojox.json.ref.serializeFunctions = true; // Persevere supports persisted functions

dojo.declare("dojox.data.PersevereStore",dojox.data.JsonRestStore,{
	_toJsonQuery: function(args){

		// performs conversion of Dojo Data query objects and sort arrays to JSONQuery strings
		if(args.query && typeof args.query == "object"){
			// convert Dojo Data query objects to JSONQuery
			var jsonQuery = "[?(", first = true;
			for(var i in args.query){
				if(args.query[i]!="*"){ // full wildcards can be ommitted
					jsonQuery += (first ? "" : "&") + "@[" + dojo._escapeString(i) + "]=" + dojox.json.ref.toJson(args.query[i]);
					first = false;
				}
			}
			if(!first){
				// use ' instead of " for quoting in JSONQuery, and end with ]
				jsonQuery += ")]"; 
			}else{
				jsonQuery = "";
			}
			args.queryStr = jsonQuery.replace(/\\"|"/g,function(t){return t == '"' ? "'" : t;});
		}else if(!args.query || args.query == '*'){
			args.query = "";
		}
		
		var sort = args.sort;
		if(sort){
			// if we have a sort order, add that to the JSONQuery expression
			args.queryStr = args.queryStr || (typeof args.query == 'string' ? args.query : ""); 
			first = true;
			for(i = 0; i < sort.length; i++){
				args.queryStr += (first ? '[' : ',') + (sort[i].descending ? '\\' : '/') + "@[" + dojo._escapeString(sort[i].attribute) + "]";
				first = false; 
			}
			if(!first){
				args.queryStr += ']';
			}
		}
		if(typeof args.queryStr == 'string'){
			args.queryStr = args.queryStr.replace(/\\"|"/g,function(t){return t == '"' ? "'" : t;});
			return args.queryStr;
		}
		return args.query;
	},
	fetch: function(args){
		this._toJsonQuery(args);
		return this.inherited(arguments);
	},
	isUpdateable: function(){
		if(!dojox.json.query){
			return this.inherited(arguments);
		}
		return true;
	},
	matchesQuery: function(item,request){
		if(!dojox.json.query){
			return this.inherited(arguments);
		}
		request._jsonQuery = request._jsonQuery || dojox.json.query(this._toJsonQuery(request)); 
		return request._jsonQuery([item]).length;
	},
	clientSideFetch: function(/*Object*/ request,/*Array*/ baseResults){
		if(!dojox.json.query){
			return this.inherited(arguments);
		}
		request._jsonQuery = request._jsonQuery || dojox.json.query(this._toJsonQuery(request));
		return request._jsonQuery(baseResults);
	},
	querySuperSet: function(argsSuper,argsSub){
		if(!dojox.json.query){
			return this.inherited(arguments);
		}
		if(!argsSuper.query){
			return argsSub.query;
		}
		return this.inherited(arguments);
	}
	
});
dojox.data.PersevereStore.getStores = function(/*String?*/path,/*Boolean?*/sync){
	// summary:
	//		Creates Dojo data stores for all the table/classes on a Persevere server
	// path:
	// 		URL of the Persevere server's root, this normally just "/"
	// 		which is the default value if the target is not provided
	// callback:
	// 		Allows the operation to happen asynchronously
	// return:
	// 		A map/object of datastores. The name of each property is a the name of a store,
	// 		and the value is the actual data store object.
	path = (path && (path.match(/\/$/) ? path : (path + '/'))) || '/';
	if(path.match(/^\w*:\/\//)){
		// if it is cross-domain, we will use window.name for communication
		dojo.require("dojox.io.xhrWindowNamePlugin");
		dojox.io.xhrWindowNamePlugin(path, dojox.io.xhrPlugins.fullHttpAdapter, true);
	}
	var rootService= dojox.rpc.Rest(path,true);
	var lastSync = dojox.rpc._sync;
	dojox.rpc._sync = sync;
	var dfd = rootService("root");//dojo.xhrGet({url: target, sync:!callback, handleAs:'json'});
	var results;
	dfd.addBoth(function(schemas){
		for(var i in schemas){
			if(typeof schemas[i] == 'object'){
				schemas[i] = new dojox.data.PersevereStore({target:new dojo._Url(path,i) + '',schema:schemas[i]});
			}
		}
		return (results = schemas);
	});
	dojox.rpc._sync = lastSync;
	return sync ? results : dfd;
};
dojox.data.PersevereStore.addProxy = function(){
	// summary:
	//		Invokes the XHR proxy plugin. Call this if you will be using x-site data.
	dojo.require("dojox.io.xhrPlugins"); // also not necessary, but we can register that Persevere supports proxying
	dojox.io.xhrPlugins.addProxy("/proxy/");
};
