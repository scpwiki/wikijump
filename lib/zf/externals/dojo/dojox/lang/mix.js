dojo.provide("dojox.lang.mix");

// This is a highly experimental functionality,
// it will be split later on individual files,
// and mixed with OO package

// TODO: do we need to use hasOwnProperty() everywhere?

(function(){
	var empty = {}, mix = dojox.lang.mix;
	
	mix.processProps = function(props, rename, remove){
		// summary: process properties in place by renaming and removing them as needed,
		//			only own properties are processed.
		// description: properties are filtered by "skip" parameter, then renamed.
		// props: Object: the object to be processed
		// rename: Object?: the dictionary for renaming
		// remove: Array?|Object?: the source of properties to be skipped
		
		if(props){
			var t, i, j, l;
			// delete properties
			if(remove){
				if(dojo.isArray(remove)){
					for(j = 0, l = remove.length; j < l; ++j){
						delete props[remove[j]];
					}
				}else{
					for(var i in remove){
						if(remove.hasOwnProperty(i)){
							delete props[i];
						}
					}
				}
			}

			// rename properties
			if(rename){
				for(i in rename){
					if(rename.hasOwnProperty(i) && props.hasOwnProperty(i)){
						t = props[i];
						delete props[i];
						props[rename[i]] = t;
					}
				}
			}
		}
		return props;	// Object
	};
	
	// preprocessing mixins
	
	var Filter = function(value, rename, skip){
		// summary: filters an object by skipping properties and renaming them
		// value: Object: object to be filtered
		// rename: Object: the dictionary for renaming
		// skip: Array?|Object?: the source of properties to be skipped
		this.value = value;
		this.rename = rename || empty;
		if(skip && dojo.isArray(skip)){
			var p = {};
			for(j = 0, l = skip.length; j < l; ++j){
				p[skip[j]] = 1;
			}
			this.skip = p;
		}else{
			this.skip = skip || empty;
		}
	};
	dojo.extend(Filter, {
		filter: function(name){
			// summary: takes in name, and filters it according to criteria
			// name: String: an input name
			// returns: String: can be the same name, new name,
			// or an empty string (when it should be deleted
			if(this.skip.hasOwnProperty(name)){ return ""; }
			return this.rename.hasOwnProperty(name) ? this.rename[name] : name;
		}
	});
	
	// mixing in members with decorators
	
	var Decorator = function(value){
		// summary: this class serves as a base class for all decorators
		// value: Object: a property to be mixed in later
		this.value = value;
	};
	dojo.extend(Decorator, {
		process: function(target, name){
			// summary: the default processing
			// target: Object: target object to accept a property
			// name: String: property's name
			if(this.value instanceof Decorator){
				this.value.process(target, name);
			}else{
				target[name] = this.value;
			}
		}
	});
	
	/*
	 * Example:
	 *  |	mix.mixer(target,
	 *  |		mix.augment(someDefaultProperties),
	 *  |		mix.filter(someOtherProperties, {exec: "doIt"}),
	 *  |		{
	 *  |			constructor: function(a, x){
	 *  |				// we assume that constructors are chained automatically
	 *  |				this.x = x;
	 *  |			},
	 *  |			advance: function(n){
	 *  |				// this is normal undecorated method
	 *  |				this.x += n;
	 *  |			},
	 *  |			log: mix.chainAfter(function(){
	 *  |				// this function will be called after the previous log()
	 *  |				console.log("x = ", this.x);
	 *  |			}),
	 *  |			debug: mix.augment(function(){
	 *  |				// this function will be added only if it was not defined yet
	 *  |				console.debug(this);
	 *  |			}),
	 *  |			display: mix.override(function(){
	 *  |				// this function will replace a previously defined display(),
	 *  |				// otherwise it is not added
	 *  |
	 *  |				// ignore the call, and do nothing
	 *  |			})
	 *  |		}
	 *  |	);
	 */

	mix.mixer = function(target, source){
		// summary: mixes two objects processing decorators
		// target: Object: target to receive new/updated properties
		// source: Object...: source of properties, more than one source is allowed
		// returns: Object: target
		
		// mixer can apply decorators by default for predefined names
		// (it doesn't do it now)
		// examples (see predefined decorators below):
		//	- undecorated "constructor" can be treated similar to chainAfter
		//	- undecorated "destroy" can be treated as chainBefore

		var dcr = null, flt = null, i, l = arguments.length, name, prop, targetName;
		for(i = 1, l; i < l; ++i){
			source = arguments[i];
			if(source instanceof Decorator){
				// use the instance of decorator as a default processor
				dcr = source;
				// extract the real source
				source = dcr.value;
			}
			if(source instanceof Filter){
				flt = source;
				source = flt.value;
			}
			for(name in source){
				if(source.hasOwnProperty(name)){
					prop = source[name];
					targetName = flt ? flt.filter(name) : name;
					if(!targetName){
						// skip filtered out names
						continue;
					}
					if(prop instanceof Decorator){
						// decorator: process it
						prop.process(target, targetName);
					}else{
						// no decorator: use the default
						if(dcr){
							dcr.value = prop;
							dcr.process(target, targetName);
						}else{
							target[targetName] = prop;
						}
					}
				}
			}
			if(flt){
				source = flt;
				flt = null;
			}
			if(dcr){
				dcr.value = source;
				dcr = null;
			}
		}
		return target;	// Object
	};

	mix.makeFilter = function(mixin){
		// summary: subclasses Filter and returns new class
		// mixin: Object: new methods
		// returns: Object: new class
		
		// don't do it at home!
		dojo.declare("dojox.__temp__", Filter, mixin || empty);
		var t = dojox.__temp__;
		delete dojox.__temp__;
		return t;	// Object
	};

	mix.createFilter = function(filter){
		// summary: simple helper function to construct decorator creators
		// filter: Function: the processing function
		// returns: Function
		var Filter = mix.makeFilter(filter && {filter: filter} || empty);
		return function(value){ return new Filter(value); };	// Function
	};

	mix.makeDecorator = function(mixin){
		// summary: subclasses Decorator and returns new class
		// mixin: Object: new methods
		// returns: Object: new class

		// don't do it at home!
		dojo.declare("dojox.__temp__", Decorator, mixin || empty);
		var t = dojox.__temp__;
		delete dojox.__temp__;
		return t;	// Object
	};

	mix.createDecorator = function(process){
		// summary: simple helper function to construct decorator creators
		// process: Function: the processing function
		// returns: Function
		var Decorator = mix.makeDecorator(process && {process: process} || empty);
		return function(value){ return new Decorator(value); };	// Function
	};
	
	// predefined decorators

	var ReplaceContextDecorator = mix.makeDecorator({
		constructor: function(context, value){
			this.value = value;	// override value
			this.context = context;
		},
		process: function(target, name){
			// summary: call the method in different context
			// passing the original "this", original arguments,
			// method's name, and the old value as 4 arguments
			// target: Object: target object to accept a property
			// name: String: property's name
			var old = target[name], current = this.value, context = this.context;
			target[name] = function(){
				return current.call(context, this, arguments, name, old);
			};
		}
	});

	// TODO: should we check for properties to be actually functions?

	dojo.mixin(mix, {
		// defalt renaming and skipping filter
		filter: mix.createFilter(),
		// conditional mixing
		augment: mix.createDecorator(function(target, name){
			// summary: add property, if it was not defined before
			if(!(name in target)){ target[name] = this.value; }
		}),
		override: mix.createDecorator(function(target, name){
			// summary: override property only if it was already present
			if(name in target){ target[name] = this.value; }
		}),
		// substitutions
		replaceContext: function(context, value){
			// summary: call the method in different context
			// passing the original "this", original arguments,
			// method's name, and the old value as 4 arguments
			// context: Object: context for the call
			// value: Function: function to call
			return new ReplaceContextDecorator(context, value);
		},
		shuffle: mix.createDecorator(function(target, name){
			// summary: replaces arguments for old method
			if(name in target){
				var old = target[name], current = this.value;
				target[name] = function(){
					return old.apply(this, current.apply(this, arguments));
				};
			}
		}),
		// chaining (see AOP mixing below for another way to do it)
		chainBefore: mix.createDecorator(function(target, name){
			// summary: creates a chain of calls where the new method is called
			// before the old method
			if(name in target){
				var old = target[name], current = this.value;
				target[name] = function(){
					current.apply(this, arguments);
					return old.apply(this, arguments);
				};
			}else{
				target[name] = this.value;
			}
		}),
		chainAfter: mix.createDecorator(function(target, name){
			// summary: creates a chain of calls where the new method is called
			// after the old method
			if(name in target){
				var old = target[name], current = this.value;
				target[name] = function(){
					old.apply(this, arguments);
					return current.apply(this, arguments);
				};
			}else{
				target[name] = this.value;
			}
		}),
		// light-weight AOP mixing
		before: mix.createDecorator(function(target, name){
			// summary: creates a "before" advise
			var old = target[name], before = this.value;
			target[name] = old ? 
				function(){
					before.apply(this, arguments);
					return old.apply(this, arguments);
				} :
				function(){
					before.apply(this, arguments);
				};
		}),
		around: mix.createDecorator(function(target, name){
			// summary: creates an "around" advise,
			// the previous value is passed as a first argument and can be "unknown",
			// arguments are passed as a second argument
			var old = target[name], around = this.value;
			target[name] = old ?
				function(){
					return around.call(this, old, arguments);
				} :
				function(){
					return around.call(this, null, arguments);
				};
		}),
		afterReturning: mix.createDecorator(function(target, name){
			// summary: creates an "afterReturning" advise,
			// the returned value is passed as the only argument
			var old = target[name], afterReturning = this.value;
			target[name] = old ?
				function(){
					var ret = old.apply(this, arguments);
					afterReturning.call(this, ret);
					return ret;
				} :
				function(){
					afterReturning.call(this);
				};
		}),
		afterThrowing: mix.createDecorator(function(target, name){
			// summary: creates an "afterThrowing" advise,
			// the exception is passed as the only argument
			var old = target[name], afterThrowing = this.value;
			if(old){
				target[name] = function(){
					var ret;
					try{
						ret = old.apply(this, arguments);
					}catch(e){
						afterThrowing.call(this, e);
						throw e;
					}
					return ret;
				};
			}
		}),
		after: mix.createDecorator(function(target, name){
			// summary: creates an "after" advise,
			// it takes no arguments
			var old = target[name], after = this.value;
			target[name] = old ?
				function(){
					var ret;
					try{
						ret = old.apply(this, arguments);
					}finally{
						after.call(this);
					}
					return ret;
				} :
				function(){
					after.call(this);
				}
		})
	});

	/*
		TODO: more possible decorators for the Mixer:
			- AOP-related:
				- concept checks
				- logging/debugging
				- argument manipulation
				- wormholing
				- memoizing
			- renaming properties to avoid conflicts
			- add complimentary properties
			- special treatment of certain names (e.g., event names starting with "on")
	*/
})();
