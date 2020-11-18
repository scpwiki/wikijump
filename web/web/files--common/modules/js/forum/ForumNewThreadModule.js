

Wikijump.modules.ForumNewThreadModule = {};

Wikijump.modules.ForumNewThreadModule.listeners = {
	cancel: function(e){
		window.location.href=Wikijump.cancelurl;
	},
	preview: function(e){
		var p = OZONE.utils.formToArray("new-thread-form");
		OZONE.ajax.requestModule("forum/ForumPreviewPostModule", p, Wikijump.modules.ForumNewThreadModule.callbacks.preview);
	},
	post: function(e){
		var p = OZONE.utils.formToArray("new-thread-form");
		p['action'] = "ForumAction";
		p['event'] = "newThread";
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Creating new thread...";
		w.show();
		OZONE.ajax.requestModule("Empty", p, Wikijump.modules.ForumNewThreadModule.callbacks.post);
	}
}

Wikijump.modules.ForumNewThreadModule.callbacks = {
	preview: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
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
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Thread has been created.";
		w.show();

		var threadId = r.threadId;
		var uri = '/forum/t-'+r.threadId+'/'+r.threadUnixifiedTitle;
		setTimeout('window.location = "'+uri+'"', 1000);
	}
}

Wikijump.modules.ForumNewThreadModule.init = function(){
	YAHOO.util.Event.addListener("ntf-cancel", "click", Wikijump.modules.ForumNewThreadModule.listeners.cancel);
	YAHOO.util.Event.addListener("ntf-preview", "click", Wikijump.modules.ForumNewThreadModule.listeners.preview);
	YAHOO.util.Event.addListener("ntf-post", "click", Wikijump.modules.ForumNewThreadModule.listeners.post);

	OZONE.dom.onDomReady(function(){
		Wikijump.Editor.init("post-edit", "post-edit-panel");
		var limiter = new OZONE.forms.lengthLimiter("thread-description", "desc-charleft", 1000);
	}, "dummy-ondomready-block");
}

Wikijump.modules.ForumNewThreadModule.init();
