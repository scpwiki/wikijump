/*
 * Wikidot - free wiki collaboration software
 * Copyright (c) 2008, Wikidot Inc.
 * 
 * Code licensed under the GNU Affero General Public 
 * License version 3 or later.
 *
 * For more information about licensing visit:
 * http://www.wikidot.org/license
 */

WIKIDOT.modules.PageTagsModule = {};

WIKIDOT.modules.PageTagsModule.listeners = {
	save: function(e){
		var p = new Object();
		p.tags = $("page-tags-input").value;
		p.pageId =  WIKIREQUEST.info.pageId;
		p.action = "WikiPageAction";
		p.event = "saveTags";
		
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving tags...";
		w.show();
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.PageTagsModule.callbacks.save);
		YAHOO.util.Event.stopEvent(e);
	}	
	
}

WIKIDOT.modules.PageTagsModule.callbacks = {
	save: function(r){
		
		if(r.status == "form_errors"){
			$("page-tags-errors").style.display = "block";
			$("page-tags-errors").innerHTML = r.message;
			return;
		}
		$("page-tags-errors").style.display = "none";
		if(!WIKIDOT.utils.handleError(r)) {return;}	
		
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Tags saved!";
		w.show();
		setTimeout('window.location.href="/'+WIKIREQUEST.info.requestPageName+'"',1500);
	}	
	
}
