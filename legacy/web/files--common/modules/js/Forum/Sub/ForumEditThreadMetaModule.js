

Wikijump.modules.ForumEditThreadMetaModule = {}

Wikijump.modules.ForumEditThreadMetaModule.listeners = {
	save: function(e){
		var p = OZONE.utils.formToArray("thread-meta-form");
		p.action = 'ForumAction';
		p.event = 'saveThreadMeta';
		OZONE.ajax.requestModule(null, p, Wikijump.modules.ForumEditThreadMetaModule.callbacks.save);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();
	}
}

Wikijump.modules.ForumEditThreadMetaModule.callbacks = {
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
		if(!Wikijump.utils.handleError(r)) {return;}

		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Your changes have been saved.";
		w.show();

		setTimeout('window.location.hash=""; window.location.reload()', 1000);

	}
}
