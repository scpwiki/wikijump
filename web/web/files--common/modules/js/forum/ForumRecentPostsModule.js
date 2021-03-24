

Wikijump.modules.ForumRecentPostsModule = {};

Wikijump.modules.ForumRecentPostsModule.listeners = {
	updateList: function(pageNo){
		var p = new Object();
		if(pageNo != null){
			p.page = pageNo;
		}else{
			p.page = 1;
		}

		p.categoryId = $("recent-posts-category").value;

		//Wikijump.modules.PageHistoryModule.vars.params = p; // for pagination

		OZONE.ajax.requestModule("Forum/ForumRecentPostsListModule", p, Wikijump.modules.ForumRecentPostsModule.callbacks.updateList);
	}
}

Wikijump.modules.ForumRecentPostsModule.callbacks = {
	updateList: function(r){
		if(!Wikijump.utils.handleError(r)) {return;}

		$("forum-recent-posts-list").innerHTML = r.body;
		OZONE.utils.formatDates("forum-recent-posts-list");
		//OZONE.dialog.hovertip.makeTip($("forum-recent-posts-list").getElementsByTagName('span'),
	}

}

Wikijump.modules.ForumRecentPostsModule.init = function(){

}

Wikijump.modules.ForumRecentPostsModule.init();
