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

WIKIDOT.modules.ForumEditThreadMetaModule = {}

WIKIDOT.modules.ForumEditThreadMetaModule.listeners = {
	save: function(e){
		var p = OZONE.utils.formToArray("thread-meta-form");
		p.action = 'ForumAction';
		p.event = 'saveThreadMeta';
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ForumEditThreadMetaModule.callbacks.save);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();
	}	
}

WIKIDOT.modules.ForumEditThreadMetaModule.callbacks = {
	save: function(r){
		if(r.status=="form_errors"){
			OZONE.dialog.cleanAll();
			var inner = "The data you have submitted contains following errors:" +
					"<ul>";
			
			var errors = r.formErrors;
			for(var i in errors){
				inner += "<li>"+errors[i]+"</li>";
			}		
			inner += "</ul>";
			$("thread-meta-errors").innerHTML = inner;
			$("thread-meta-errors").style.display = "block";
			return;
		}
		if(!WIKIDOT.utils.handleError(r)) {return;}	
		
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Your changes have been saved.";
		w.show();
		
		setTimeout('window.location.hash=""; window.location.reload()', 1000);
		
	}	
}