

Wikijump.modules.PageBlockModule = {};

Wikijump.modules.PageBlockModule.listeners = {
	save: function(event){
		var p = new Object();
		p.pageId = WIKIREQUEST.info.pageId;
		if($("page-block-checkbox").checked){
			p.block = true;
		}
		p.action = 'WikiPageAction';
		p.event = 'saveBlock';
		OZONE.ajax.requestModule(null, p, Wikijump.modules.PageBlockModule.callbacks.save);
		var w = new OZONE.dialogs.WaitBox();
		w.content = "Saving changes...";
		w.show();
	}
}

Wikijump.modules.PageBlockModule.callbacks = {
	save: function(r){
		if(r.status != 'ok'){
			var w = new OZONE.dialogs.ErrorDialog();	w.content = r.message; w.show();	return;
		}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Your changes have been saved.";
		w.show();

		setTimeout('window.location.href="/'+WIKIREQUEST.info.requestPageName+'"',1500);

	}
}
