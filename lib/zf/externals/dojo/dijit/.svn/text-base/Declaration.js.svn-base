dojo.provide("dijit.Declaration");
dojo.require("dijit._Widget");
dojo.require("dijit._Templated");

dojo.declare(
	"dijit.Declaration",
	dijit._Widget,
	{
		// summary:
		//		The Declaration widget allows a user to declare new widget
		//		classes directly from a snippet of markup.

		_noScript: true,
		widgetClass: "",
		replaceVars: true,
		defaults: null,
		mixins: [],
		buildRendering: function(){
			var src = this.srcNodeRef.parentNode.removeChild(this.srcNodeRef),
				preambles = dojo.query("> script[type='dojo/method'][event='preamble']", src).orphan(),
				methods = dojo.query("> script[type^='dojo/method'][event]", src).orphan(),
				postscriptConnects = dojo.query("> script[type^='dojo/method']", src).orphan(),
				regularConnects = dojo.query("> script[type^='dojo/connect']", src).orphan(),
				srcType = src.nodeName;

			var propList = this.defaults||{};

			// For all methods defined like <script type="dojo/method" event="foo">,
			// add that method to prototype
			dojo.forEach(methods, function(s){
				var evt = s.getAttribute("event"),
					func = dojo.parser._functionFromScript(s);
				propList[evt] = func;
			});

			// map array of strings like [ "dijit.form.Button" ] to array of mixin objects
			// (note that dojo.map(this.mixins, dojo.getObject) doesn't work because it passes
			// a bogus third argument to getObject(), confusing it)
			this.mixins = this.mixins.length ?
				dojo.map(this.mixins, function(name){ return dojo.getObject(name); } ) :
				[ dijit._Widget, dijit._Templated ];

			propList.widgetsInTemplate = true;
			propList._skipNodeCache = true;
			propList.templateString = "<"+srcType+" class='"+src.className+"' dojoAttachPoint='"+(src.getAttribute("dojoAttachPoint")||'')+"' dojoAttachEvent='"+(src.getAttribute("dojoAttachEvent")||'')+"' >"+src.innerHTML.replace(/\%7B/g,"{").replace(/\%7D/g,"}")+"</"+srcType+">";

			// strip things so we don't create stuff under us in the initial setup phase
			dojo.query("[dojoType]", src).forEach(function(node){
				node.removeAttribute("dojoType");
			});

			// create the new widget class
			var wc = dojo.declare(
				this.widgetClass,
				this.mixins,
				propList
			);

			// Handle <script> blocks of form:
			//		<script type="dojo/connect" event="foo">
			// and
			//		<script type="dojo/method">
			// (Note that the second one is just shorthand for a dojo/connect to postscript)
			// Since this is a connect in the declaration, we are actually connection to the method
			// in the _prototype_.
			var connects = regularConnects.concat(postscriptConnects);
			dojo.forEach(connects, function(s){
				var evt = s.getAttribute("event")||"postscript",
					func = dojo.parser._functionFromScript(s);
				dojo.connect(wc.prototype, evt, func);
			});
		}
	}
);
