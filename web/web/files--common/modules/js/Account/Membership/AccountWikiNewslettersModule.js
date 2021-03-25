

Wikijump.modules.AccountWikiNewslettersModule = {};

Wikijump.modules.AccountWikiNewslettersModule.listeners = {
	checkAll: function(e, value){
		var inps = YAHOO.util.Dom.getElementsByClassName("receive-newsletter", "input", "receive-wiki-newsletters-form");
		for(var i=0; i<inps.length; i++){
			inps[i].checked = value;
		}
	},

	saveDefault: function(e){
		var p = new Object();
		p.action =
	}

}

Wikijump.modules.AccountWikiNewslettersModule.callbacks = {
	saveDefault: function(r){

	}
}
