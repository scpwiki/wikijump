

Wikijump.modules.NewPageHelperModule = {};

Wikijump.modules.NewPageHelperModule.listeners = {
	create: function(e){
		YAHOO.util.Event.stopEvent(e);
		var f = YAHOO.util.Event.getTarget(e);
		while(f && !f.tagName.match(/^form$/i)){
			f = f.parentNode;
		}
		if(!f){
			return;
		}
		var ts = f.getElementsByTagName("select")[0];
		if(ts && ts.value == ''){
			alert("Please select a template.");
			return;
		}
		var p = OZONE.utils.formToArray(f);
		p.action = 'misc/NewPageHelperAction';
		p.event = 'createNewPage';
		OZONE.ajax.requestModule(null, p, Wikijump.modules.NewPageHelperModule.callbacks.create);
		return false;
	}
}

Wikijump.modules.NewPageHelperModule.callbacks = {
	create: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		// go to page and edit it!
		var href =  "/"+r.unixName+'/edit/true';
		if(r.templateId){
			href += '/t/'+r.templateId;
		}
		if(r.pageTitle){
			href += '/title/' + encodeURIComponent(r.pageTitle);
		}
		window.location.href = href;
	}
}
