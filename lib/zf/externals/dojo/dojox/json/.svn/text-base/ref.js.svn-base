dojo.provide("dojox.json.ref");
dojo.require("dojo.date.stamp");

// summary:
// Adds advanced JSON {de}serialization capabilities to the base json library.
// This enhances the capabilities of dojo.toJson and dojo.fromJson,
// adding referencing support, date handling, and other extra format handling.
// On parsing, references are resolved. When references are made to
// ids/objects that have been loaded yet, the loader function will be set to
// _loadObject to denote a lazy loading (not loaded yet) object. 

dojox.json.ref = {
	resolveJson: function(/*Object*/ root,/*Object?*/ args){
		// summary:
		// 		Indexes and resolves references in the JSON object.
		// description:
		// 		A JSON Schema object that can be used to advise the handling of the JSON (defining ids, date properties, urls, etc)
		//
		// root:
		//		The root object of the object graph to be processed
		// args:
		//		Object with additional arguments:
		//
		// The *index* parameter.
		//		This is the index object (map) to use to store an index of all the objects. 
		// 		If you are using inter-message referencing, you must provide the same object for each call.
		// The *defaultId* parameter.
		//		This is the default id to use for the root object (if it doesn't define it's own id)
		//	The *idPrefix* parameter.
		//		This the prefix to use for the ids as they enter the index. This allows multiple tables 
		// 		to use ids (that might otherwise collide) that enter the same global index. 
		// 		idPrefix should be in the form "/Service/".  For example,
		//		if the idPrefix is "/Table/", and object is encountered {id:"4",...}, this would go in the
		//		index as "/Table/4".
		//	The *idAttribute* parameter.
		//		This indicates what property is the identity property. This defaults to "id"
		//	The *assignAbsoluteIds* parameter.
		//		This indicates that the resolveJson should assign absolute ids (__id) as the objects are being parsed.
		//  
		// The *schemas* parameter
		//		This provides a map of schemas, from which prototypes can be retrieved
		// The *loader* parameter
		//		This is a function that is called added to the reference objects that can't be resolved (lazy objects)
		// return:
		//		An object, the result of the processing
		args = args || {};
		var idAttribute = args.idAttribute || 'id';
		var prefix = args.idPrefix || '/'; 
		var assignAbsoluteIds = args.assignAbsoluteIds;
		var index = args.index || {}; // create an index if one doesn't exist
		var ref,reWalk=[];
		var pathResolveRegex = /^(.*\/)?(\w+:\/\/)|[^\/\.]+\/\.\.\/|^.*\/(\/)/;
		var addProp = this._addProp;
		function walk(it, stop, defaultId, defaultObject){
			// this walks the new graph, resolving references and making other changes
		 	var update, val, id = it[idAttribute] || defaultId;
		 	if(id !== undefined){
		 		id = (prefix + id).replace(pathResolveRegex,'$2$3');
		 	}
		 	var target = defaultObject || it;
			if(id !== undefined){ // if there is an id available...
				if(assignAbsoluteIds){
					it.__id = id;
				}
				// if the id already exists in the system, we should use the existing object, and just 
				// update it... as long as the object is compatible
				if(index[id] && ((it instanceof Array) == (index[id] instanceof Array))){ 
					target = index[id];
					delete target.$ref; // remove this artifact
					update = true;
				}else{
				 	var proto = args.schemas && (!(it instanceof Array)) && // won't try on arrays to do prototypes, plus it messes with queries 
		 					(val = id.match(/^(.+\/)[^\.\[]*$/)) && // if it has a direct table id (no paths)
		 					(val = args.schemas[val[1]]) && val.prototype; // and if has a prototype
					if(proto){
						// if the schema defines a prototype, that needs to be the prototype of the object
						var F = function(){};
						F.prototype = proto;
						target = new F();
					}
				}
				index[id] = target; // add the prefix, set _id, and index it
			}
	
	
			for(var i in it){
				if(it.hasOwnProperty(i)){
					if((typeof (val=it[i]) =='object') && val){
						ref=val.$ref;
						if(ref){ // a reference was found
							var stripped = ref.replace(/\\./g, '@').replace(/"[^"\\\n\r]*"/g, '');// trim it
							if(/[\w\[\]\.\$# \/\r\n\t]/.test(stripped) && !/\=|((^|\W)new\W)/.test(stripped)){
								// make sure it is a safe reference
								delete it[i];// remove the property so it doesn't resolve to itself in the case of id.propertyName lazy values
								var path = ref.match(/(^([^\[]*\/)?[^\.\[]*)([\.\[].*)?/); // divide along the path
								if((ref = (path[1]=='$' || path[1]=='this' || path[1]=='#') ? root : index[(prefix + path[1]).replace(pathResolveRegex,'$2$3')])){  // a $ indicates to start with the root, otherwise start with an id
									// // starting point was found, use eval to resolve remaining property references
									// // need to also make reserved words safe by replacing with index operator
									try{
										ref = path[3] ? eval('ref' + path[3].replace(/^#/,'').replace(/^([^\[\.])/,'.$1').replace(/\.([\w$_]+)/g,'["$1"]')) : ref;
									}catch(e){
										ref = null;
									}
								}
								if(ref){
									// otherwise, no starting point was found (id not found), if stop is set, it does not exist, we have
									// unloaded reference, if stop is not set, it may be in a part of the graph not walked yet,
									// we will wait for the second loop
									val = ref;
								}else{
									if(!stop){
										var rewalking;
										if(!rewalking){
											reWalk.push(target); // we need to rewalk it to resolve references
										}
										rewalking = true; // we only want to add it once
									}else{
										val = walk(val, false, val.$ref);
										// create a lazy loaded object
										val._loadObject = args.loader;
									}
								}
							}
						}else{
							if(!stop){ // if we are in stop, that means we are in the second loop, and we only need to check this current one,
								// further walking may lead down circular loops
								val = walk(
									val,
									reWalk==it,
									id && addProp(id, i), // the default id to use 
									// if we have an existing object child, we want to 
									// maintain it's identity, so we pass it as the default object
									target != it && typeof target[i] == 'object' && target[i] 
								);
							}
						}
					}
					it[i] = val;
					if(target!=it){// performance guard				
						var old = target[i];
						target[i] = val; // update the target
						if(update && val !== old){ // only update if it changed
							if(index.onUpdate){
								index.onUpdate(target,i,old,val); // call the listener for each update
							}
						}
					}
				}
			}
	
			if(update){
				// this means we are updating, we need to remove deleted
				for(i in target){
					if(!it.hasOwnProperty(i) && i != '__id' && i != '__clientId' && !(target instanceof Array && isNaN(i))){
						if(index.onUpdate){
							index.onUpdate(target,i,target[i],undefined); // call the listener for each update
						}
						delete target[i];
						while(target instanceof Array && target.length && target[target.length-1] === undefined){
							// shorten the target if necessary
							target.length--;
						}
					}
				}
			}else{
				if(index.onLoad){
					index.onLoad(target);
				}
			}
			return target;
		}
		if(root && typeof root == 'object'){
			root = walk(root,false,args.defaultId); // do the main walk through
			walk(reWalk,false); // re walk any parts that were not able to resolve references on the first round
		}
		return root;
	},


	fromJson: function(/*String*/ str,/*Object?*/ args){
	// summary:
	// 		evaluates the passed string-form of a JSON object.
	//
	// str:
	//		a string literal of a JSON item, for instance:
	//			'{ "foo": [ "bar", 1, { "baz": "thud" } ] }'
	// args: See resolveJson
	//
	// return:
	//		An object, the result of the evaluation
		function ref(target){ // support call styles references as well
			return {$ref:target};
		}
		var root = eval('(' + str + ')'); // do the eval
		if(root){
			return this.resolveJson(root, args);
		}
		return root;
	},
	
	toJson: function(/*Object*/ it, /*Boolean?*/ prettyPrint, /*Object?*/ idPrefix, /*Object?*/ indexSubObjects){
		// summary:
		//		Create a JSON serialization of an object.
		//		This has support for referencing, including circular references, duplicate references, and out-of-message references
		// 		id and path-based referencing is supported as well and is based on http://www.json.com/2007/10/19/json-referencing-proposal-and-library/.
		//
		// it:
		//		an object to be serialized.
		//
		// prettyPrint:
		//		if true, we indent objects and arrays to make the output prettier.
		//		The variable dojo.toJsonIndentStr is used as the indent string
		//		-- to use something other than the default (tab),
		//		change that variable before calling dojo.toJson().
		//
		// idPrefix: The prefix that has been used for the absolute ids
		//
		// return:
		//		a String representing the serialized version of the passed object.
		var useRefs = this._useRefs;
		var addProp = this._addProp;
		idPrefix = idPrefix || ''; // the id prefix for this context
		var paths=indexSubObjects || {};
		function serialize(it,path,_indentStr){
			if(typeof it == 'object' && it){
				var value;
				if(it instanceof Date){ // properly serialize dates
					return '"' + dojo.date.stamp.toISOString(it,{zulu:true}) + '"';
				}
				var id = it.__id;
				if(id){ // we found an identifiable object, we will just serialize a reference to it... unless it is the root
					if(path != '#' && (useRefs || paths[id])){
						var ref = id; // a pure path based reference, leave it alone
	
						if(id.charAt(0)!='#'){
							if(id.substring(0, idPrefix.length) == idPrefix){ // see if the reference is in the current context
								// a reference with a prefix matching the current context, the prefix should be removed
								ref = id.substring(idPrefix.length);
							}else{
								// a reference to a different context, assume relative url based referencing
								ref = id;
							}
						}
						return serialize({
							$ref: ref
						},'#');
					}
					path = id;
				}else{
					it.__id = path; // we will create path ids for other objects in case they are circular
					paths[path] = it;// save it here so they can be deleted at the end
				}
				_indentStr = _indentStr || "";
				var nextIndent = prettyPrint ? _indentStr + dojo.toJsonIndentStr : "";
				var newLine = prettyPrint ? "\n" : "";
				var sep = prettyPrint ? " " : "";
	
				if(it instanceof Array){
					var res = dojo.map(it, function(obj,i){
						var val = serialize(obj, addProp(path, i), nextIndent);
						if(typeof val != "string"){
							val = "undefined";
						}
						return newLine + nextIndent + val;
					});
					return "[" + res.join("," + sep) + newLine + _indentStr + "]";
				}
	
				var output = [];
				for(var i in it){
					if(it.hasOwnProperty(i)){
						var keyStr;
						if(typeof i == "number"){
							keyStr = '"' + i + '"';
						}else if(typeof i == "string" && i.charAt(0) != '_'){
							keyStr = dojo._escapeString(i);
						}else{
							// skip non-string or number keys
							continue;
						}
						var val = serialize(it[i],addProp(path, i),nextIndent);
						if(typeof val != "string"){
							// skip non-serializable values
							continue;
						}
						output.push(newLine + nextIndent + keyStr + ":" + sep + val);
					}
				}
				return "{" + output.join("," + sep) + newLine + _indentStr + "}";
			}else if(typeof it == "function" && dojox.json.ref.serializeFunctions){
				return it.toString();
			}
	
			return dojo.toJson(it); // use the default serializer for primitives
		}
		var json = serialize(it,'#','');
		if(!indexSubObjects){
			for(i in paths)  {// cleanup the temporary path-generated ids
				delete paths[i].__id;
			}
		}
		return json;
	},
	_addProp: function(id, prop){
		return id + (id.match(/#/) ? '' : '#') +
					(typeof prop == 'string' ? // is it a string
						prop.match(/^[a-zA-Z]\w*$/) ? ('.' + prop) : // yes, otherwise we have to escape it
							('[' + dojo._escapeString(prop).replace(/"/g,"'") + ']') :
						('[' + prop + ']'));
	},
	_useRefs: false,
	serializeFunctions: false
}
