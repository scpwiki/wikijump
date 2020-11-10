

Wikijump.modules.UserRecentPostsModule = {};

Wikijump.modules.UserRecentPostsModule.listeners = {
	updateList: function(pageNo){
		var p = new Object();
		if(pageNo != null){
			p.page = pageNo;
		}else{
			p.page = 1;
		}

		p.userId = USERINFO.userId;

		//Wikijump.modules.PageHistoryModule.vars.params = p; // for pagination

		OZONE.ajax.requestModule("userinfo/UserRecentPostsListModule", p, Wikijump.modules.UserRecentPostsModule.callbacks.updateList);
	}
}

Wikijump.modules.UserRecentPostsModule.callbacks = {
	updateList: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		$("forum-recent-posts-list").innerHTML = r.body;
		OZONE.utils.formatDates("forum-recent-posts-list");
		//OZONE.dialog.hovertip.makeTip($("forum-recent-posts-list").getElementsByTagName('span'),
	}

}

Wikijump.modules.UserRecentPostsModule.init = function(){

}

Wikijump.modules.UserRecentPostsModule.init();
