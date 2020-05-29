dojo.provide("dojox.rpc.JsonRest");

dojo.require("dojox.json.ref"); // this provides json indexing
dojo.require("dojox.rpc.Rest");
// summary:
// 		Provides JSON/REST utility functions
(function(){
	var dirtyObjects = [];
	var Rest = dojox.rpc.Rest;
	var parentIdRegex = /(.*?)(#?(\.\w+)|(\[.+))+$/;
	var jr = dojox.rpc.JsonRest={
		commit: function(kwArgs){
			// summary:
			//		Saves the dirty data using REST Ajax methods

			var left; // this is how many changes are remaining to be received from the server
			kwArgs = kwArgs || {};
			var actions = [];
			var alreadyRecorded = {};
			var savingObjects = [];
			for(var i = 0; i < dirtyObjects.length; i++){
				var dirty = dirtyObjects[i];
				var object = dirty.object;
				var old = dirty.old;
				var append = false;
				if(!(kwArgs.service && (object || old) && (object || old).__id.indexOf(kwArgs.service.servicePath))){
					if(object){
						if(old){
							// changed object
							while(!(dojox.json && dojox.json.ref && dojox.json.ref._useRefs) && object.__id.match(parentIdRegex)){ // it is a path reference
								// this means it is a sub object and the server doesn't support directly putting to
								// this object by path, we must go to the parent object and save it
								var parentId = object.__id.match(parentIdRegex)[1];
								// record that we are saving
								object = Rest._index[parentId];
							}
							if(!(object.__id in alreadyRecorded)){// if it has already been saved, we don't want to repeat it
								alreadyRecorded[object.__id] = object;
								actions.push({method:"put",target:object,content:object});
							}
						}else{
							// new object
							
							actions.push({method:"post",target:{__id:jr.getServiceAndId(object.__id).service.servicePath},
													content:object});
						}
					}else if(old){
						// deleted object
						actions.push({method:"delete",target:old});
					}//else{ this would happen if an object is created and then deleted, don't do anything
					savingObjects.push(dirty);
					dirtyObjects.splice(i--,1);
				}
			}
			var xhrSendId;
			var plainXhr = dojo.xhr;
			left = actions.length;
			var contentLocation;
			// add headers for extra information
			dojo.xhr = function(method,args){
				// keep the transaction open as we send requests
				args.headers = args.headers || {};
				// the last one should commit the transaction
				args.headers['X-Transaction'] = actions.length - 1 == i ? "commit" : "open";
				if(contentLocation){
					args.headers['Content-Location'] = contentLocation;
				}
				return plainXhr.apply(dojo,arguments);
			};			
			for(i =0; i < actions.length;i++){ // iterate through the actions to execute
				var action = actions[i];
				dojox.rpc.JsonRest._contentId = action.content && action.content.__id; // this is used by OfflineRest
				var isPost = action.method == 'post';
				// send the content location to the server
				contentLocation = isPost && dojox.rpc.JsonRest._contentId;
				var serviceAndId = jr.getServiceAndId(action.target.__id);
				var service = serviceAndId.service; 
				var dfd = action.deferred = service[action.method](
									serviceAndId.id.replace(/#/,''), // if we are using references, we need eliminate #
									dojox.json.ref.toJson(action.content, false, service.servicePath, true)
								);
				(function(object, dfd, service){
					dfd.addCallback(function(value){
						try{
							// Implements id assignment per the HTTP specification
							var newId = dfd.ioArgs.xhr.getResponseHeader("Location");
							//TODO: match URLs if the servicePath is relative...
							if(newId){
								// if the path starts in the middle of an absolute URL for Location, we will use the just the path part 
								var startIndex = newId.match(/(^\w+:\/\/)/) && newId.indexOf(service.servicePath);
								newId = startIndex > 0 ? newId.substring(startIndex) : (service.servicePath + newId).
										// now do simple relative URL resolution in case of a relative URL. 
										replace(/^(.*\/)?(\w+:\/\/)|[^\/\.]+\/\.\.\/|^.*\/(\/)/,'$2$3');
								object.__id = newId;
								Rest._index[newId] = object;
							}
							value = value && dojox.json.ref.resolveJson(value, {
								index: Rest._index,
								idPrefix: service.servicePath,
								idAttribute: jr.getIdAttribute(service),
								schemas: jr.schemas,
								loader:	jr._loader,
								assignAbsoluteIds: true
							});
						}catch(e){}
						if(!(--left)){
							if(kwArgs.onComplete){
								kwArgs.onComplete.call(kwArgs.scope);
							}
						}
						return value;
					});
				})(isPost && action.content, dfd, service);
								
				dfd.addErrback(function(value){
					
					// on an error we want to revert, first we want to separate any changes that were made since the commit
					left = -1; // first make sure that success isn't called
					var postCommitDirtyObjects = dirtyObjects;
					dirtyObject = savingObjects;
					numDirty = 0; // make sure this does't do anything if it is called again
					jr.revert(); // revert if there was an error
					dirtyObjects = postCommitDirtyObjects;
					if(kwArgs.onError){
						kwArgs.onError();
					}
					return value;
				});
			}
			// revert back to the normal XHR handler
			dojo.xhr = plainXhr;
			
			return actions;
		},
		getDirtyObjects: function(){
			return dirtyObjects;
		},
		revert: function(){
			// summary:
			//		Reverts all the changes made to JSON/REST data
			while (dirtyObjects.length>0){
				var d = dirtyObjects.pop();
				if(d.object && d.old){
					// changed
					for(var i in d.old){
						if(d.old.hasOwnProperty(i)){
							d.object[i] = d.old[i];
						}
					}
					for(i in d.object){
						if(!d.old.hasOwnProperty(i)){
							delete d.object[i];
						}
					}
				}
			}
		},
		changing: function(object,_deleting){
			// summary:
			//		adds an object to the list of dirty objects.  This object
			//		contains a reference to the object itself as well as a
			//		cloned and trimmed version of old object for use with
			//		revert.
			if(!object.__id){
				return;
			}
			//if an object is already in the list of dirty objects, don't add it again
			//or it will overwrite the premodification data set.
			for(var i=0; i<dirtyObjects.length; i++){
				if(object==dirtyObjects[i].object){
					if(_deleting){
						// we are deleting, no object is an indicator of deletiong
						dirtyObjects[i].object = false;
					}
					return;
				}
			}
			var old = object instanceof Array ? [] : {};
			for(i in object){
				if(object.hasOwnProperty(i)){
					old[i] = object[i];
				}
			}
			dirtyObjects.push({object: !_deleting && object, old: old});
		},
		deleteObject: function(object){
			// summary:
			//		deletes an object 
			//	object:
			//  	object to delete
			//

			this.changing(object,true);
		},
		getConstructor: function(/*Function|String*/service, schema){
			// summary:
			// 		Creates or gets a constructor for objects from this service
			if(typeof service == 'string'){
				var servicePath = service;
				service = new dojox.rpc.Rest(service,true);
				this.registerService(service, servicePath, schema);
			}
			if(service._constructor){
				return service._constructor;
			}
			service._constructor = function(data){
				// summary:
				//		creates a new object for this table
				//
				//	data:
				//		object to mixed in
				if(data){
					dojo.mixin(this,data);
				}
				var idAttribute = jr.getIdAttribute(service);
				Rest._index[this.__id = this.__clientId = service.servicePath + (this[idAttribute] || (this[idAttribute] = Math.random().toString(16).substring(2,14)+Math.random().toString(16).substring(2,14)))] = this;
				dirtyObjects.push({object:this});
	//			this._getParent(parentInfo).push(data); // append to this list

			};
			return dojo.mixin(service._constructor, service._schema, {load:service});
		},
		fetch: function(absoluteId){
			// summary:
			//		Fetches a resource by an absolute path/id and returns a dojo.Deferred.
			var serviceAndId = jr.getServiceAndId(absoluteId);
			return this.byId(serviceAndId.service,serviceAndId.id);
		},
		getIdAttribute: function(service){
			// summary:
			//		Return the ids attribute used by this service (based on it's schema).
			//		Defaults to "id", if not other id is defined
			var schema = service._schema;
			var idAttr;
			if(schema){
				if(!(idAttr = schema._idAttr)){
					for(var i in schema.properties){
						if(schema.properties[i].identity){
							schema._idAttr = idAttr = i;
						}
					}
				}
			}
			return idAttr || 'id';
		},
		getServiceAndId: function(/*String*/absoluteId){
			// summary:
			//		Returns the REST service and the local id for the given absolute id. The result 
			// 		is returned as an object with a service property and an id property
			//	absoluteId:
			//		This is the absolute id of the object
			var parts = absoluteId.match(/^(.*\/)([^\/]*)$/);
			var svc = jr.services[parts[1]] || new dojox.rpc.Rest(parts[1], true); // use an existing or create one
			return { service: svc, id:parts[2] };
		},
		services:{},
		schemas:{},
		registerService: function(/*Function*/ service, /*String*/ servicePath, /*Object?*/ schema){
			//	summary:
			//		Registers a service for as a JsonRest service, mapping it to a path and schema
			//	service:
			//		This is the service to register
			//	servicePath:
			//		This is the path that is used for all the ids for the objects returned by service
			//	schema:
			//		This is a JSON Schema object to associate with objects returned by this service
			servicePath = servicePath || service.servicePath;
			servicePath = service.servicePath = servicePath.match(/\/$/) ? servicePath : (servicePath + '/'); // add a trailing / if needed
			service._schema = jr.schemas[servicePath] = schema || service._schema || {};
			jr.services[servicePath] = service;
		},
		byId: function(service, id){
			// if caching is allowed, we look in the cache for the result
			var deferred, result = Rest._index[(service.servicePath || '') + id];
			if(result && !result._loadObject){// cache hit
				deferred = new dojo.Deferred();
				deferred.callback(result);
				return deferred;
			}
			return this.query(service, id);
		},
		query: function(service, id, args){
			var deferred = service(id, args);
			deferred.addCallback(function(result){
				if(result.nodeType && result.cloneNode){
					// return immediately if it is an XML document
					return result;
				}
				return dojox.json.ref.resolveJson(result, {
					defaultId: typeof id != 'string' || (args && (args.start || args.count)) ? undefined: id, 
					index: Rest._index,
					idPrefix: service.servicePath,
					idAttribute: jr.getIdAttribute(service),
					schemas: jr.schemas,
					loader:	jr._loader,
					assignAbsoluteIds: true
				});
			});
			return deferred;			
		},
		_loader: function(callback){
			// load a lazy object
			var serviceAndId = jr.getServiceAndId(this.__id);
			var self = this;
			jr.query(serviceAndId.service, serviceAndId.id).addBoth(function(result){
				// if they are the same this means an object was loaded, otherwise it 
				// might be a primitive that was loaded or maybe an error
				if(result == self){
					// we can clear the flag, so it is a loaded object
					delete result.$ref;
					delete result._loadObject;
				}else{
					// it is probably a primitive value, we can't change the identity of an object to
					//	the loaded value, so we will keep it lazy, but define the lazy loader to always
					//	return the loaded value
					self._loadObject = function(callback){
						callback(result);
					};
				}
				callback(result);
			});
		},
		isDirty: function(item){
			// summary
			//		returns true if the item is marked as dirty.
			for(var i = 0, l = dirtyObjects.length; i < l; i++){
				if(dirtyObjects[i].object==item){return true;}
			}
			return false;
		}
	};
})();


