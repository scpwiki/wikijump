dojo.provide("demos.i18n.model");
dojo.declare(
	"i18nTreeModel",
	null,
{
	// summary:
	//		Implements dijit.Tree.model connecting to I18N data

	// stub to call geoname RPC server
	geoname: null,
	
	// array of info about each country
	countries: null,

	constructor: function(args){
		this.lang = args.lang || "en";

		// This allows calls to geonames server
		this.geonames = new dojox.rpc.Service(dojo.moduleUrl("dojox.rpc.SMDLibrary", "geonames.smd"));
	},

	destroy: function(){
	},

	// =======================================================================
	// Methods for traversing hierarchy

	getRoot: function(onItem, onError){
		onItem({type: "root", root: true, name: "Continents"});
	},

	mayHaveChildren: function(/*dojo.data.Item*/ item){
		// summary:
		//		Tells if an item has or may have children.
		return item.root || item.type == "continent" || item.type == "country";
	},

	getChildren: function(/*dojo.data.Item*/ item, /*function(items)*/ onComplete, /*function*/ onError){
		// summary:
		// 		Calls onComplete() with array of child items of given parent item, all loaded.

		switch(item.root ?  "top" : item.type){
			case "top":
				var d = dojo.xhrGet({url: "continents.json", handleAs: "json-comment-optional"});
				d.addCallback(onComplete);
				break;
			case "continent":
				// Unfortunately we can't query countries filtering by continent, so the first time
				// we hit this code we just query all the countries and store them away
				var callback = function(countries){
					var matches = dojo.filter(countries, function(c){
						c.type = "country";
						return c.continent == item.iso;
					});
					onComplete( matches );
				};
				
				if(!this.countries){
					// Load country database over RPC to Geonames server

					var def = this.geonames.getCountryInfo({lang: this.lang});
					def.addCallback(this, function(result){
						this.countries = result.geonames;
						callback(this.countries);
					});
				}else{
					callback(this.countries);
				}
				break;
			case "country":
				// First time we are asked for the children of a country, need to load
				// the mapping from language code --> language name in the current page's locale
				if(!this.d2){
					this.d2 = dojo.xhrGet({url: "languages.json", handleAs: "json-comment-optional"});
					this.d2.addCallback(this, function(data){
						// first item on deferred chain filters languages.json data down to
						// language names in current page's locale
						return dojo.filter(data, function(l){
							return l.iso == this.lang;
						})[0] || {};
					});
				}
				// Once we have mapping from language ISO code to language name, use it
				this.d2.addCallback(function(mapping){
					// summary:
					//		Converts a comma separated list of locales into
					//		an array of locale objects, and passes that array to onComplete

					// split localeList into array, ignoring final blank entry due to trailing comma
					var ary = dojo.filter(item.languages.split(","), function(loc){
						return loc;
					});

					// map array into locale objects
					var locales = dojo.map(ary, function(loc){
						loc = loc.toLowerCase();

						// get language object for the page language
						var langISO = loc.replace(/-.*/, "");

						// get localized name of language
						var name = mapping[langISO] || "";

						return {
							type: "locale",
							iso: loc,
							lang: langISO,
							name: name + " \u202b(" + loc + ")\u202c"
						};
					});
					onComplete(locales); 
					return mapping;		// Deferreds need to return their original arg for the next guy in chain (in the future)
				});
				break;
		}
	},

	// =======================================================================
	// Inspecting items

	getIdentity: function(/* item */ item){
		return item.iso || item.countryCode;
	},

	getLabel: function(/*dojo.data.Item*/ item){
		// summary: get the label for an item
		return item.name || item.countryName;
	},

	// =======================================================================
	// Write interface (unimplemented)

	newItem: function(/* Object? */ args, /*Item*/ parent){
	},

	pasteItem: function(/*Item*/ childItem, /*Item*/ oldParentItem, /*Item*/ newParentItem, /*Boolean*/ bCopy){
	},

	// =======================================================================
	// Callbacks (unimplemented)
	
	onChange: function(/*dojo.data.Item*/ item){
	},

	onChildrenChange: function(/*dojo.data.Item*/ parent, /*dojo.data.Item[]*/ newChildrenList){
	},

	onDelete: function(/*dojo.data.Item*/ parent, /*dojo.data.Item[]*/ newChildrenList){
	}
});