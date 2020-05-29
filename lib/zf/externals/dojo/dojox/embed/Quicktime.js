dojo.provide("dojox.embed.Quicktime");

(function(){
	/*******************************************************
		dojox.embed.Quicktime

		Base functionality to insert a QuickTime movie
		into a document on the fly.
	 ******************************************************/

	var qtMarkup, qtVersion, installed, __def__={
		width: 320,
		height: 240,
		redirect: null
	};
	var keyBase="dojox-embed-quicktime-", keyCount=0;

	//	reference to the test movie we will use for getting QT info from the browser.
	var testMovieUrl=dojo.moduleUrl("dojox", "embed/resources/version.mov");

	//	*** private methods *********************************************************
	function prep(kwArgs){
		kwArgs = dojo.mixin(dojo.clone(__def__), kwArgs || {});
		if(!("path" in kwArgs)){
			console.error("dojox.embed.Quicktime(ctor):: no path reference to a QuickTime movie was provided.");
			return null;
		}
		if(!("id" in kwArgs)){
			kwArgs.id=(keyBase + keyCount++);
		}
		return kwArgs;
	}
	
	var getQTMarkup = 'This content requires the <a href="http://www.apple.com/quicktime/download/" title="Download and install QuickTime.">QuickTime plugin</a>.';
	if(dojo.isIE){
		qtVersion = 0;
		installed = (function(){
			try{
				var o = new ActiveXObject("QuickTimeCheckObject.QuickTimeCheck.1");
				if(o!==undefined){
					//	pull the qt version too
					var v=o.QuickTimeVersion.toString(16);
					qtVersion={
						major: parseInt(v.substring(0,1),10)||0,
						minor: parseInt(v.substring(1,2),10)||0,
						rev: parseInt(v.substring(2,3), 10)||0
					};
					return o.IsQuickTimeAvailable(0);
				}
			} catch(e){ }
			return false;
		})();

		qtMarkup = function(kwArgs){
			if(!installed){ return { id: null, markup: getQTMarkup }; }
			
			kwArgs = prep(kwArgs);
			if(!kwArgs){ return null; }
			var s = '<object classid="clsid:02BF25D5-8C17-4B23-BC80-D3488ABDDC6B" '
				+ 'codebase="http://www.apple.com/qtactivex/qtplugin.cab#version=6,0,2,0" '
				+ 'id="' + kwArgs.id + '" '
				+ 'width="' + kwArgs.width + '" '
				+ 'height="' + kwArgs.height + '">'
				+ '<param name="src" value="' + kwArgs.path + '" />';
			if(kwArgs.params){
				for(var p in kwArgs.params){
					s += '<param name="' + p + '" value="' + kwArgs.params[p] + '" />';
				}
			}
			s += '</object>';
			return { id: kwArgs.id, markup: s };
		}
	} else {
		installed = (function(){
			for(var i=0, l=navigator.plugins.length; i<l; i++){
				if(navigator.plugins[i].name.indexOf("QuickTime")>-1){
					return true;
				}
			}
			return false;
		})();

		qtMarkup = function(kwArgs){
			if(!installed){ return { id: null, markup: getQTMarkup }; }

			kwArgs = prep(kwArgs);
			if(!kwArgs){ return null; }
			var s = '<embed type="video/quicktime" src="' + kwArgs.path + '" '
				+ 'id="' + kwArgs.id + '" '
				+ 'name="' + kwArgs.id + '" '
				+ 'pluginspage="www.apple.com/quicktime/download" '
				+ 'enablejavascript="true" '
				+ 'width="' + kwArgs.width + '" '
				+ 'height="' + kwArgs.height + '"';
			if(kwArgs.params){
				for(var p in kwArgs.params){
					s += ' ' + p + '="' + kwArgs.params[p] + '"';
				}
			}
			s += '></embed>';
			return { id: kwArgs.id, markup: s };
		}
	}

	/*=====
	dojox.embed.__QTArgs = function(path, id, width, height, params, redirect){
		//	path: String
		//		The URL of the movie to embed.
		//	id: String?
		//		A unique key that will be used as the id of the created markup.  If you don't
		//		provide this, a unique key will be generated.
		//	width: Number?
		//		The width of the embedded movie; the default value is 320px.
		//	height: Number?
		//		The height of the embedded movie; the default value is 240px
		//	params: Object?
		//		A set of key/value pairs that you want to define in the resultant markup.
		//	redirect: String?
		//		A url to redirect the browser to if the current QuickTime version is not supported.
		this.id=id;
		this.path=path;
		this.width=width;
		this.height=height;
		this.params=params;
		this.redirect=redirect;
	}
	=====*/

	dojox.embed.Quicktime=function(/* dojox.embed.__QTArgs */kwArgs, /* DOMNode */node){
		//	summary:
		//		Returns a reference to the HTMLObject/HTMLEmbed that is created to 
		//		place the movie in the document.  You can use this either with or
		//		without the new operator.  Note that with any other DOM manipulation,
		//		you must wait until the document is finished loading before trying
		//		to use this.
		//
		//	example:
		//		Embed a QuickTime movie in a document using the new operator, and get a reference to it.
		//	|	var movie = new dojox.embed.QuickTime({
		//	|		path: "path/to/my/movie.mov",
		//	|		width: 400,
		//	|		height: 300
		//	|	}, myWrapperNode);
		//
		//	example:
		//		Embed a movie in a document without using the new operator.
		//	|	var movie = dojox.embed.QuickTime({
		//	|		path: "path/to/my/movie.mov",
		//	|		width: 400,
		//	|		height: 300
		//	|	}, myWrapperNode);

		return dojox.embed.Quicktime.place(kwArgs, node);	//	HTMLObject
	};

	dojo.mixin(dojox.embed.Quicktime, {
		//	summary:
		//		A singleton object used internally to get information
		//		about the QuickTime player available in a browser, and
		//		as the factory for generating and placing markup in a
		//		document.
		//
		//	minSupported: Number
		//		The minimum supported version of the QuickTime Player, defaults to
		//		6.
		//	available: Boolean
		//		Whether or not QuickTime is available.
		//	supported: Boolean
		//		Whether or not the QuickTime Player installed is supported by
		//		dojox.embed.
		//	version: Object
		//		The version of the installed QuickTime Player; takes the form of
		//		{ major, minor, rev }.  To get the major version, you'd do this:
		//		var v=dojox.embed.Quicktime.version.major;
		//	initialized: Boolean
		//		Whether or not the QuickTime engine is available for use.
		//	onInitialize: Function
		//		A stub you can connect to if you are looking to fire code when the 
		//		engine becomes available.  A note: do NOT use this stub to embed
		//		a movie in your document; this WILL be fired before DOMContentLoaded
		//		is fired, and you will get an error.  You should use dojo.addOnLoad
		//		to place your movie instead.

		minSupported: 6,
		available: installed,
		supported: installed,
		version: qtVersion,
		initialized: false,
		onInitialize: function(){ 
			dojox.embed.Quicktime.initialized = true; 
		},	//	stub function to let you know when this is ready

		place: function(kwArgs, node){
			var o = qtMarkup(kwArgs);

			node = dojo.byId(node);
			if(!node){
				node=dojo.doc.createElement("div");
				node.id=o.id+"-container";
				dojo.body().appendChild(node);
			}
			
			if(o){
				node.innerHTML = o.markup;
				if(o.id){
					return (dojo.isIE)? dojo.byId(o.id) : document[o.id];	//	QuickTimeObject
				}
			}
			return null;	//	QuickTimeObject
		}
	});

	//	go get the info
	if(!dojo.isIE){
		// FIXME: Opera does not like this at all for some reason, and of course there's no event references easily found.
		qtVersion = dojox.embed.Quicktime.version = { major: 0, minor: 0, rev: 0 };
		var o = qtMarkup({ path: testMovieUrl, width:4, height:4 });

		function qtInsert(){
			if(!dojo._initFired){
				var s='<div style="top:0;left:0;width:1px;height:1px;;overflow:hidden;position:absolute;" id="-qt-version-test">'
					+ o.markup
					+ '</div>';
				document.write(s);
			} else {
				var n = document.createElement("div");
				n.id="-qt-version-test";
				n.style.cssText = "top:0;left:0;width:1px;height:1px;overflow:hidden;position:absolute;";
				dojo.body().appendChild(n);
				n.innerHTML = o.markup;
			}
		}

		function qtGetInfo(mv){
			var qt, n, v = [ 0, 0, 0 ];
			if(mv){
				qt=mv, n=qt.parentNode;
			} else {
				if(o.id) {
					qtInsert();
					if(!dojo.isOpera){
						setTimeout(function(){ qtGetInfo(document[o.id]); }, 50);
					} else {
						var fn=function(){ 
							setTimeout(function(){ qtGetInfo(document[o.id]) }, 50); 
						};
						if(!dojo._initFired){
							dojo.addOnLoad(fn);
						} else {
							dojo.connect(document[o.id], "onload", fn);
						}
					}
				}
				return;
			}

			if(qt){
				try {
					v = qt.GetQuickTimeVersion().split(".");
					qtVersion = { major: parseInt(v[0]||0), minor: parseInt(v[1]||0), rev: parseInt(v[2]||0) };
				} catch(e){ 
					qtVersion = { major: 0, minor: 0, rev: 0 };
				}
			}

			dojox.embed.Quicktime.supported = v[0];
			dojox.embed.Quicktime.version = qtVersion;
			if(dojox.embed.Quicktime.supported){
				dojox.embed.Quicktime.onInitialize();
			} else {
				console.log("quicktime is not installed.");
			}

			try {
				if(!mv){
					dojo.body().removeChild(n);
				}
			} catch(e){ }
		}

		qtGetInfo();
	}
	else if(dojo.isIE && installed){
		dojox.embed.Quicktime.onInitialize();
	}
})();
