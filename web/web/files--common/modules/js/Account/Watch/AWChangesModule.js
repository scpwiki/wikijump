

Wikijump.modules.AWChangesModule = {};

Wikijump.modules.AWChangesModule.listeners = {
	showWatchedPages: function(e){
		OZONE.ajax.requestModule("/Account/Watch/AWPagesListModule", null, Wikijump.modules.AWChangesModule.callbacks.showWatchedPages);
	},
	hideWatchedPages: function(e){
		$("watched-pages-list").innerHTML="";
		$("watched-pages-list").style.display = "none";
		$("show-watched-pages-button").style.display="";
		$("hide-watched-pages-button").style.display="none";
	},

	removeWatchedPage: function(e, pageId){
		var p = new Object();
		p.pageId = pageId;
		p.action = "WatchAction";
		p.event = "removeWatchedPage";
		OZONE.ajax.requestModule(null, p, Wikijump.modules.AWChangesModule.callbacks.removeWatchedPage);
	},
	updateList: function(event,pageNo){
		var p = new Object();
		if(pageNo != null){
			p.page = pageNo;
		}else{
			p.page = 1;
		}

		OZONE.ajax.requestModule("Account/Watch/AWChangesListModule", p, Wikijump.modules.AWChangesModule.callbacks.updateList);
	}
}

Wikijump.modules.AWChangesModule.callbacks = {
	showWatchedPages: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var el = $("watched-pages-list");
		el.innerHTML = r.body;
		el.style.display = "block";
		$("hide-watched-pages-button").style.display="";
		$("show-watched-pages-button").style.display="none";
	},

	removeWatchedPage: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}
		var w = new OZONE.dialogs.SuccessBox();
		w.content = "Page not being watched any more.";
		w.show();
		Wikijump.modules.AWChangesModule.listeners.showWatchedPages();
		Wikijump.modules.AWChangesModule.listeners.updateList();
	},
	updateList: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		$("watched-changes-list").innerHTML = r.body;
		OZONE.utils.formatDates("watched-changes-list");
		OZONE.dialog.hovertip.makeTip($("watched-changes-list").getElementsByTagName('span'),
				{style: {width: 'auto'}});
	}

}

Wikijump.modules.AWChangesModule.init = function(){
		OZONE.utils.formatDates("watched-changes-list");
		OZONE.dialog.hovertip.makeTip($("watched-changes-list").getElementsByTagName('span'),
				{style: {width: 'auto'}});
}

Wikijump.modules.AWChangesModule.init();
