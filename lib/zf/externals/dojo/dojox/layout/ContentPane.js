dojo.provide("dojox.layout.ContentPane");

dojo.require("dijit.layout.ContentPane");
dojo.require("dojox.html._base"); 

(function(){ // private scope, sort of a namespace

	/*=====
	dojox.layout.ContentPane.DeferredHandle = {
		// cancel: Function
		cancel: function(){
			// summary: cancel a in flight download
		},

		addOnLoad: function(func){
			// summary: add a callback to the onLoad chain
			// func: Function
		},

		addOnUnload: function(func){
			// summary: add a callback to the onUnload chain
			// func: Function
		}
	}
	=====*/


dojo.declare("dojox.layout.ContentPane", dijit.layout.ContentPane, {
	// summary:
	//		An extended version of dijit.layout.ContentPane
	//		Supports infile scripts and external ones declared by <script src=''
	//		relative path adjustments (content fetched from a different folder)
	//		<style> and <link rel='stylesheet' href='..'> tags,
	//		css paths inside cssText is adjusted (if you set adjustPaths = true)
	//
	//		NOTE that dojo.require in script in the fetched file isn't recommended
	//		Many widgets need to be required at page load to work properly

	// adjustPaths: Boolean
	//		Adjust relative paths in html string content to point to this page
	//		Only usefull if you grab content from a another folder then the current one
	adjustPaths: false,

	// cleanContent: Boolean
	//	summary:
	//		cleans content to make it less likly to generate DOM/JS errors.
	//	description:
	//		useful if you send contentpane a complete page, instead of a html fragment
	//		scans for 
	//
	//			* style nodes, inserts in Document head
	//			* title Node, remove
	//			* DOCTYPE tag, remove
	//			* `<!-- *JS code here* -->`
	//			* `<![CDATA[ *JS code here* ]]>`
	cleanContent: false,

	// renderStyles: Boolean
	//		trigger/load styles in the content
	renderStyles: false,

	// executeScripts: Boolean
	//		Execute (eval) scripts that is found in the content
	executeScripts: true,

	// scriptHasHooks: Boolean
	//		replace keyword '_container_' in scripts with 'dijit.byId(this.id)'
	// NOTE this name might change in the near future
	scriptHasHooks: false,

	/*======
	// ioMethod: dojo.xhrGet|dojo.xhrPost
	//		reference to the method that should grab the content
	ioMethod: dojo.xhrGet,
	
	// ioArgs: Object
	//		makes it possible to add custom args to xhrGet, like ioArgs.headers['X-myHeader'] = 'true'
	ioArgs: {},

	// onLoadDeferred: dojo.Deferred
	//		callbackchain will start when onLoad occurs
	onLoadDeferred: new dojo.Deferred(),

	// onUnloadDeferred: dojo.Deferred
	//		callbackchain will start when onUnload occurs
	onUnloadDeferred: new dojo.Deferred(),

	setHref: function(url){
		// summary: replace current content with url's content
		return ;// dojox.layout.ContentPane.DeferredHandle
	},

	refresh: function(){
		// summary: force a re-download of content
		return ;// dojox.layout.ContentPane.DeferredHandle 
	},

	======*/

	constructor: function(){
		// init per instance properties, initializer doesn't work here because how things is hooked up in dijit._Widget
		this.ioArgs = {};
		this.ioMethod = dojo.xhrGet;
		this.onLoadDeferred = new dojo.Deferred();
		this.onUnloadDeferred = new dojo.Deferred();
	},

	postCreate: function(){
		// override to support loadDeferred
		this._setUpDeferreds();

		dijit.layout.ContentPane.prototype.postCreate.apply(this, arguments);
	},

	onExecError: function(e){
		// summary
		//		event callback, called on script error or on java handler error
		//		overide and return your own html string if you want a some text 
		//		displayed within the ContentPane
	},

	_setContentAttr: function(data){
		var defObj = this._setUpDeferreds();
		this.inherited(arguments);
		return defObj; // dojox.layout.ContentPane.DeferredHandle
	},

	cancel: function(){
		// summary: cancels a inflight download
		if(this._xhrDfd && this._xhrDfd.fired == -1){
			// we are still in flight, which means we should reset our DeferredHandle
			// otherwise we will trigger onUnLoad chain of the canceled content,
			// the canceled content have never gotten onLoad so it shouldn't get onUnload
			this.onUnloadDeferred = null;
		}
		dijit.layout.ContentPane.prototype.cancel.apply(this, arguments);
	},

	_setUpDeferreds: function(){
		var _t = this, cancel = function(){ _t.cancel();	};
		var onLoad = (_t.onLoadDeferred = new dojo.Deferred());
		var onUnload = (_t._nextUnloadDeferred = new dojo.Deferred());
		return {
			cancel: cancel,
			addOnLoad: function(func){onLoad.addCallback(func);},
			addOnUnload: function(func){onUnload.addCallback(func);}
		};
	},

	_onLoadHandler: function(){
		dijit.layout.ContentPane.prototype._onLoadHandler.apply(this, arguments);
		if(this.onLoadDeferred){
			this.onLoadDeferred.callback(true);
		}
	},

	_onUnloadHandler: function(){
		this.isLoaded = false;
		this.cancel();// need to cancel so we don't get any inflight suprises
		if(this.onUnloadDeferred){
			this.onUnloadDeferred.callback(true);
		}

		dijit.layout.ContentPane.prototype._onUnloadHandler.apply(this, arguments);

		if(this._nextUnloadDeferred){
			this.onUnloadDeferred = this._nextUnloadDeferred;
		}
	},

	_onError: function(type, err){
		dijit.layout.ContentPane.prototype._onError.apply(this, arguments);
		if(this.onLoadDeferred){
			this.onLoadDeferred.errback(err);
		}
	},

	_prepareLoad: function(forceLoad){
		// sets up for a xhrLoad, load is deferred until widget is showing
		var defObj = this._setUpDeferreds();

		dijit.layout.ContentPane.prototype._prepareLoad.apply(this, arguments);

		return defObj;
	},

	_setContent: function(cont){
		// override dijit.layout.ContentPane._setContent, to enable path adjustments
		
		var setter = this._contentSetter; 
		if(! (setter && setter instanceof dojox.html._ContentSetter)) {
			setter = this._contentSetter = new dojox.html._ContentSetter({
				node: this.containerNode,
				_onError: dojo.hitch(this, this._onError),
				onContentError: dojo.hitch(this, function(e){
					// fires if a domfault occurs when we are appending this.errorMessage
					// like for instance if domNode is a UL and we try append a DIV
					var errMess = this.onContentError(e);
					try{
						this.containerNode.innerHTML = errMess;
					}catch(e){
						console.error('Fatal '+this.id+' could not change content due to '+e.message, e);
					}
				})/*,
				_onError */
			});
		};

		// stash the params for the contentSetter to allow inheritance to work for _setContent
		this._contentSetterParams = {
			adjustPaths: Boolean(this.adjustPaths && (this.href||this.referencePath)),
			referencePath: this.href || this.referencePath,
			renderStyles: this.renderStyles,
			executeScripts: this.executeScripts,
			scriptHasHooks: this.scriptHasHooks,
			scriptHookReplacement: "dijit.byId('"+this.id+"')"
		};

		this.inherited("_setContent", arguments);
	}
	// could put back _renderStyles by wrapping/aliasing dojox.html._ContentSetter.prototype._renderStyles
});

})();
