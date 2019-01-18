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

WIKIDOT.modules.ForumRecentPostsModule = {};

WIKIDOT.modules.ForumRecentPostsModule.listeners = {
	updateList: function(pageNo){
		var p = new Object();
		if(pageNo != null){
			p.page = pageNo;
		}else{
			p.page = 1;
		}
		
		p.categoryId = $("recent-posts-category").value;

		//WIKIDOT.modules.PageHistoryModule.vars.params = p; // for pagination
		
		OZONE.ajax.requestModule("forum/ForumRecentPostsListModule", p, WIKIDOT.modules.ForumRecentPostsModule.callbacks.updateList);
	}
}

WIKIDOT.modules.ForumRecentPostsModule.callbacks = {
	updateList: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		$("forum-recent-posts-list").innerHTML = r.body;
		OZONE.utils.formatDates("forum-recent-posts-list");
		//OZONE.dialog.hovertip.makeTip($("forum-recent-posts-list").getElementsByTagName('span'),
	}	
	
}

WIKIDOT.modules.ForumRecentPostsModule.init = function(){
	
}

WIKIDOT.modules.ForumRecentPostsModule.init();
