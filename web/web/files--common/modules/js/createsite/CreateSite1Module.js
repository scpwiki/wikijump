

Wikijump.modules.CreateSite1Module = {};

Wikijump.modules.CreateSite1Module.listeners = {
	cancelClick: function(e){
		window.location.href="/";
	},

	nextClick: function(e){
		parms = new Array();
		parms['action']='CreateSiteAction';
		parms['event']='finalize';
		OZONE.ajax.requestModule("CreateSite/CreateSite2Module", parms, Wikijump.modules.CreateSite1Module.callbacks.nextClick);

	},

	backClick: function(e){
		OZONE.ajax.requestModule("CreateSite/CreateSite0Module", null, Wikijump.modules.CreateSite1Module.callbacks.backClick);

	}

}
Wikijump.modules.CreateSite1Module.callbacks = {
	nextClick: function(response){
		OZONE.utils.setInnerHTMLContent("create-site-area", response.body);

	},
	backClick: function(response){
		OZONE.utils.setInnerHTMLContent("create-site-area", response.body);

	}

}
