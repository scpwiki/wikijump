dojo.provide("dijit.robotx");
dojo.require("dijit.robot");
dojo.require("dojo.robotx");
dojo.experimental("dijit.robotx");
(function(){
var __updateDocument = doh.robot._updateDocument;

dojo.mixin(doh.robot,{
	_updateDocument: function(){
		__updateDocument();
		var win = (dojo.doc.parentWindow || dojo.doc.defaultView);
		if(win["dijit"]){
			dijit.registry = win.dijit.registry;
		}
	}
});

})();
