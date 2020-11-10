

Wikijump.modules.PageTagsModule = {};

Wikijump.modules.PageTagsModule.listeners = {
	save: function(e){
		var p = new Object();
		p.tags = $("page-tags-input").value;
		p.pageId =  WIKIREQUEST.info.pageId;
		p.action = "WikiPageAction";
		p.event = "saveTags";

		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving tags...";
		w.show();
		OZONE.ajax.requestModule(null, p, Wikijump.modules.PageTagsModule.callbacks.save);
		YAHOO.util.Event.stopEvent(e);
	}

}

Wikijump.modules.PageTagsModule.callbacks = {
	save: function(r){

		if(r.status == "form_errors"){
			$("page-tags-errors").style.display = "block";
			$("page-tags-errors").innerHTML = r.message;
			return;
		}
		$("page-tags-errors").style.display = "none";
		if(!Wikijump.utils.handleError(r)) {return;}

		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Tags saved!";
		w.show();
		setTimeout('window.location.href="/'+WIKIREQUEST.info.requestPageName+'"',1500);
	}

}
