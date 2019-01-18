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

WIKIDOT.modules.ForumNewThreadModule = {};

WIKIDOT.modules.ForumNewThreadModule.listeners = {
	cancel: function(e){
		window.location.href=WIKIDOT.cancelurl;
	},
	preview: function(e){
		var p = OZONE.utils.formToArray("new-thread-form");	
		OZONE.ajax.requestModule("forum/ForumPreviewPostModule", p, WIKIDOT.modules.ForumNewThreadModule.callbacks.preview);
	},
	post: function(e){
		var p = OZONE.utils.formToArray("new-thread-form");	
		p['action'] = "ForumAction";
		p['event'] = "newThread";
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Creating new thread...";
		w.show();
		OZONE.ajax.requestModule("Empty", p, WIKIDOT.modules.ForumNewThreadModule.callbacks.post);
	}
}

WIKIDOT.modules.ForumNewThreadModule.callbacks = {
	preview: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}	
		$("message-preview-wrapper").style.display="block";
		OZONE.utils.setInnerHTMLContent("message-preview", r.body);
		OZONE.visuals.scrollTo("message-preview");
	},
	post: function(r){
		if(r.status=="form_errors"){
			OZONE.dialog.cleanAll();
			var inner = "The data you have submitted contains following errors:" +
					"<ul>";
			
			var errors = r.formErrors;
			for(var i in errors){
				inner += "<li>"+errors[i]+"</li>";
			}		
			inner += "</ul>";
			$("new-thread-error").innerHTML = inner;
			$("new-thread-error").style.display="block";
			return;
		}
		if(!WIKIDOT.utils.handleError(r)) {return;}	
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Thread has been created.";
		w.show();
		
		var threadId = r.threadId;
		var uri = '/forum/t-'+r.threadId+'/'+r.threadUnixifiedTitle;
		setTimeout('window.location = "'+uri+'"', 1000);
	}
}

WIKIDOT.modules.ForumNewThreadModule.init = function(){
	YAHOO.util.Event.addListener("ntf-cancel", "click", WIKIDOT.modules.ForumNewThreadModule.listeners.cancel);
	YAHOO.util.Event.addListener("ntf-preview", "click", WIKIDOT.modules.ForumNewThreadModule.listeners.preview);
	YAHOO.util.Event.addListener("ntf-post", "click", WIKIDOT.modules.ForumNewThreadModule.listeners.post);
	
	OZONE.dom.onDomReady(function(){
		WIKIDOT.Editor.init("post-edit", "post-edit-panel");
		var limiter = new OZONE.forms.lengthLimiter("thread-description", "desc-charleft", 1000);
	}, "dummy-ondomready-block");
}

WIKIDOT.modules.ForumNewThreadModule.init();
