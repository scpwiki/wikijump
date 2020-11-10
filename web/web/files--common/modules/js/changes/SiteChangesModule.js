

Wikijump.modules.SiteChangesModule = {};

Wikijump.modules.SiteChangesModule.listeners = {
	updateList: function(pageNo){
		var p = new Object();
		if(pageNo != null){
			p.page = pageNo;
		}else{
			p.page = 1;
		}
		p.perpage = $("rev-perpage").value;
		p.pageId = WIKIREQUEST.info.pageId;
		p.categoryId = $("rev-category").value;

		// which revisions...
		var o = new Object();
		if($("rev-type-all").checked) o.all = true;
		if($("rev-type-source").checked) o.source = true;
		if($("rev-type-title").checked) o.title = true;
		if($("rev-type-move").checked) o.move = true;
		if($("rev-type-files").checked) o.files = true;
		if($("rev-type-new").checked) o['new'] = true;
		if($("rev-type-meta").checked) o.meta = true;

		p.options = JSON.stringify(o);

		//Wikijump.modules.PageHistoryModule.vars.params = p; // for pagination

		OZONE.ajax.requestModule("changes/SiteChangesListModule", p, Wikijump.modules.SiteChangesModule.callbacks.updateList);
	}
}

Wikijump.modules.SiteChangesModule.callbacks = {
	updateList: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		$("site-changes-list").innerHTML = r.body;
		OZONE.utils.formatDates("site-changes-list");
		OZONE.dialog.hovertip.makeTip($("site-changes-list").getElementsByTagName('span'),
				{style: {width: 'auto'}});
	}

}

Wikijump.modules.SiteChangesModule.init = function(){
	OZONE.dom.onDomReady(function(){
		OZONE.utils.formatDates("site-changes-list");
		OZONE.dialog.hovertip.makeTip($("site-changes-list").getElementsByTagName('span'),
					{style: {width: 'auto'}});
	}, "dummy-ondomready-block");
}

Wikijump.modules.SiteChangesModule.init();
