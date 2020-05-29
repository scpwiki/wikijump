dojo.provide("dojox.lang.tests.main");

try{
	// functional block
	dojo.require("dojox.lang.tests.listcomp");
	dojo.require("dojox.lang.tests.lambda");
	dojo.require("dojox.lang.tests.fold");
	dojo.require("dojox.lang.tests.curry");
	dojo.require("dojox.lang.tests.misc");
	dojo.require("dojox.lang.tests.array");
	dojo.require("dojox.lang.tests.object");
	dojo.require("dojox.lang.tests.mix");
	dojo.require("dojox.lang.tests.recomb");
	dojo.require("dojox.lang.tests.observable");
}catch(e){
	doh.debug(e);
}
