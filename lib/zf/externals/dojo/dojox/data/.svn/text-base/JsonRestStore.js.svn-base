dojo.provide("dojox.data.JsonRestStore");

dojo.require("dojox.data.ServiceStore");
dojo.require("dojox.rpc.JsonRest");


dojo.declare("dojox.data.JsonRestStore",
	dojox.data.ServiceStore,
	{
		constructor: function(options){
			//summary:
			//		JsonRestStore is a Dojo Data store interface to JSON HTTP/REST web
			//		storage services that support read and write through GET, PUT, POST, and DELETE. 
			// options: 
			// 		Keyword arguments
			//	
			// The *schema* parameter
			//		This is a schema object for this store. This should be JSON Schema format.
			//
			// The *service* parameter
			// 		This is the service object that is used to retrieve lazy data and save results
			// 		The function should be directly callable with a single parameter of an object id to be loaded
			// 		The function should also have the following methods:
			// 			put(id,value) - puts the value at the given id
			// 			post(id,value) - posts (appends) the value at the given id
			// 			delete(id) - deletes the value corresponding to the given id
			//		Note that it is critical that the service parses responses as JSON.
			//		If you are using dojox.rpc.Service, the easiest way to make sure this 
			// 		happens is to make the responses have a content type of 
			// 		application/json.
			//
			// The *target* parameter
			// 		This is the target URL for this Service store. This may be used in place
			// 		of a service parameter to connect directly to RESTful URL without
			// 		using a dojox.rpc.Service object.
			//
			// The *idAttribute* parameter
			//		Defaults to 'id'. The name of the attribute that holds an objects id.
			//		This can be a preexisting id provided by the server.
			//		If an ID isn't already provided when an object
			//		is fetched or added to the store, the autoIdentity system
			//		will generate an id for it and add it to the index.
			//
			// The *syncMode* parameter
			//		Setting this to true will set the store to using synchronous calls by default.
			//		Sync calls return their data immediately from the calling function, so
			//		callbacks are unnecessary
			//
			//	description:
			//		The JsonRestStore will then cause all saved modifications to be server using Rest commands (PUT, POST, or DELETE).
			// 		When using a Rest store on a public network, it is important to implement proper security measures to
			//		control access to resources.
			//		On the server side implementing a REST interface means providing GET, PUT, POST, and DELETE handlers.
			//		GET - Retrieve an object or array/result set, this can be by id (like /table/1) or with a 
			// 			query (like /table/?name=foo). 
			//		PUT - This should modify a object, the URL will correspond to the id (like /table/1), and the body will 
			// 			provide the modified object
			//		POST - This should create a new object. The URL will correspond to the target store (like /table/)
			// 			and the body should be the properties of the new object. The server's response should include a
			// 			Location header that indicates the id of the newly created object. This id will be used for subsequent
			// 			PUT and DELETE requests. JsonRestStore also includes a Content-Location header that indicates
			//			the temporary randomly generated id used by client, and this location is used for subsequent 
			// 			PUT/DELETEs if no Location header is provided by the server or if a modification is sent prior 
			// 			to receiving a response from the server. 
			// 		DELETE - This should delete an object by id.
			// 		These articles include more detailed information on using the JsonRestStore:
			//		http://www.sitepen.com/blog/2008/06/13/restful-json-dojo-data/
			//		http://blog.medryx.org/2008/07/24/jsonreststore-overview/
			//		
			//	example:
			// 		A JsonRestStore takes a REST service or a URL and uses it the remote communication for a
			// 		read/write dojo.data implementation. A JsonRestStore can be created with a simple URL like:
			// 	|	new JsonRestStore({target:"/MyData/"});
			//	example:
			// 		To use a JsonRestStore with a service, you should create a
			// 		service with a REST transport. This can be configured with an SMD:
			//	|	{
			//	|		services: {
			//	|			jsonRestStore: {
			//	|				transport: "REST",
			//	|				envelope: "URL",
			//	|				target: "store.php",
			//	|				contentType:"application/json",
			//	|				parameters: [
			//	|					{name: "location", type: "string", optional: true}
			//	|				]
			//	|			}
			//	|		}
			//	|	}
			// 		The SMD can then be used to create service, and the service can be passed to a JsonRestStore. For example:
			//	|	var myServices = new dojox.rpc.Service(dojo.moduleUrl("dojox.rpc.tests.resources", "test.smd"));
			//	|	var jsonStore = new dojox.data.JsonRestStore({service:myServices.jsonRestStore});
			//	example:
			//		The JsonRestStore also supports lazy loading. References can be made to objects that have not been loaded.
			//		For example if a service returned:
			//	|	{"name":"Example","lazyLoadedObject":{"$ref":"obj2"}}
			// 		And this object has accessed using the dojo.data API:
			//	|	var obj = jsonStore.getValue(myObject,"lazyLoadedObject");
			//		The object would automatically be requested from the server (with an object id of "obj2").
			//	

			dojo.connect(dojox.rpc.Rest._index,"onUpdate",this,function(obj,attrName,oldValue,newValue){
				var prefix = this.service.servicePath;
				if(!obj.__id){
					console.log("no id on updated object ", obj);
				}else if(obj.__id.substring(0,prefix.length) == prefix){
					this.onSet(obj,attrName,oldValue,newValue);
				}
			});
			this.idAttribute = this.idAttribute || 'id';// no options about it, we have to have identity
			//setup a byId alias to the api call
			if(typeof options.target == 'string' && !this.service){
				this.service = dojox.rpc.Rest(this.target,true); // create a default Rest service
			}
			dojox.rpc.JsonRest.registerService(this.service, options.target, this.schema);
			this.schema = this.service._schema = this.schema || this.service._schema || {};
			// wrap the service with so it goes through JsonRest manager 
			this.service._store = this;
			this.schema._idAttr = this.idAttribute;
			this._constructor = dojox.rpc.JsonRest.getConstructor(this.service);
			this._index = dojox.rpc.Rest._index;
			//given a url, load json data from as the store
		},
		target:"",
		//Write API Support
		newItem: function(data, parentInfo){
			// summary:
			//		adds a new item to the store at the specified point.
			//		Takes two parameters, data, and options.
			//
			//	data: /* object */
			//		The data to be added in as an item.
			data = new this._constructor(data);
			if(parentInfo){
				// get the previous value or any empty array
				var values = this.getValue(parentInfo.parent,parentInfo.attribute,[]);
				// set the new value
				this.setValue(parentInfo.parent,parentInfo.attribute,values.concat([data]));
			}
			this.onNew(data); // should this go before the set?
			return data;
		},
		deleteItem: function(item){
			// summary:
			//		deletes item any references to that item from the store.
			//
			//	item:
			//  	item to delete
			//

			//	If the desire is to delete only one reference, unsetAttribute or
			//	setValue is the way to go.
			dojox.rpc.JsonRest.deleteObject(item);
			var store = dojox.data._getStoreForItem(item);
			store.onDelete(item);
		},
		changing: function(item,_deleting){
			// summary:
			//		adds an item to the list of dirty items.  This item
			//		contains a reference to the item itself as well as a
			//		cloned and trimmed version of old item for use with
			//		revert.
			dojox.rpc.JsonRest.changing(item,_deleting);
		},

		setValue: function(item, attribute, value){
			// summary:
			//		sets 'attribute' on 'item' to 'value'

			var old = item[attribute];
			var store = item.__id ? dojox.data._getStoreForItem(item) : this;
			if(dojox.json.schema && store.schema && store.schema.properties){
				// if we have a schema and schema validator available we will validate the property change
				var result = dojox.json.schema.checkPropertyChange(value,store.schema.properties[attribute]);
				if(!result.valid){
					throw new Error(dojo.map(result.errors,function(error){return error.message;}).join(","));
				}
			}
			if(old !== value){
				store.changing(item);
				item[attribute]=value;
				store.onSet(item,attribute,old,value);
			}
		},
		setValues: function(item, attribute, values){
			// summary:
			//	sets 'attribute' on 'item' to 'value' value
			//	must be an array.


			if(!dojo.isArray(values)){
				throw new Error("setValues expects to be passed an Array object as its value");
			}
			this.setValue(item,attribute,values);
		},

		unsetAttribute: function(item, attribute){
			// summary:
			//		unsets 'attribute' on 'item'

			this.changing(item);
			var old = item[attribute];
			delete item[attribute];
			this.onSet(item,attribute,old,undefined);
		},
		save: function(kwArgs){
			// summary:
			//		Saves the dirty data using REST Ajax methods. See dojo.data.api.Write for API.
			//
			//	kwArgs.global:
			//		This will cause the save to commit the dirty data for all 
			// 		JsonRestStores as a single transaction.

			if(!(kwArgs && kwArgs.global)){
				(kwArgs = kwArgs || {}).service = this.service;
			} 
			var actions = dojox.rpc.JsonRest.commit(kwArgs);
			this.serverVersion = this._updates && this._updates.length;
			return actions;
		},

		revert: function(){
			// summary
			//		returns any modified data to its original state prior to a save();
			var dirtyObjects = dojox.rpc.JsonRest.getDirtyObjects().concat([]);
			while (dirtyObjects.length>0){
				var d = dirtyObjects.pop();
				var store = dojox.data._getStoreForItem(d.object || d.old);
				if(!d.object){
					// was a deletion, we will add it back
					store.onNew(d.old);
				}else if(!d.old){
					// was an addition, remove it
					store.onDelete(d.object);
				}else{
					// find all the properties that were modified
					for(var i in d.object){
						if(d.object[i] != d.old[i]){
							store.onSet(d.object, i, d.object[i], d.old[i]);
						}
					}
				}
			}
			dojox.rpc.JsonRest.revert();
		},

		isDirty: function(item){
			// summary
			//		returns true if the item is marked as dirty.
			return dojox.rpc.JsonRest.isDirty(item);
		},
		isItem: function(item){
			// summary:
			//	Checks to see if a passed 'item'
			//	is really belongs to this JsonRestStore.
			//
			//	item: /* object */
			//	attribute: /* string */
			return item && item.__id && this.service == dojox.rpc.JsonRest.getServiceAndId(item.__id).service;
		},
		_doQuery: function(args){
			var query= typeof args.queryStr == 'string' ? args.queryStr : args.query;
			return dojox.rpc.JsonRest.query(this.service,query, args);
		},
		_processResults: function(results, deferred){
			// index the results
			var count = results.length;
			// if we don't know the length, and it is partial result, we will guess that it is twice as big, that will work for most widgets
			return {totalCount:deferred.fullLength || (deferred.request.count == count ? count * 2 : count), items: results};
		},

		getConstructor: function(){
			// summary:
			// 		Gets the constructor for objects from this store
			return this._constructor;
		},
		getIdentity: function(item){
			var id = item.__clientId || item.__id;
			if(!id){
				this.inherited(arguments); // let service store throw the error
			}
			var prefix = this.service.servicePath;
			// support for relative or absolute referencing with ids 
			return id.substring(0,prefix.length) != prefix ?  id : id.substring(prefix.length); // String
		},
		fetchItemByIdentity: function(args){
			var id = args.identity;
			var store = this;
			// if it is an absolute id, we want to find the right store to query
			if(id.match(/^(\w*:)?\//)){
				var serviceAndId = dojox.rpc.JsonRest.getServiceAndId(id);
				store = serviceAndId.service._store;
				args.identity = serviceAndId.id; 
			}
			args._prefix = store.service.servicePath;
			return store.inherited(arguments);
		},
		//Notifcation Support

		onSet: function(){},
		onNew: function(){},
		onDelete: 	function(){},

		getFeatures: function(){
			// summary:
			// 		return the store feature set
			var features = this.inherited(arguments);
			features["dojo.data.api.Write"] = true;
			features["dojo.data.api.Notification"] = true;
			return features;
		}


	}
);
dojox.data._getStoreForItem = function(item){
	return dojox.rpc.JsonRest.services[item.__id.match(/.*\//)[0]]._store;
};
