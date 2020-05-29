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

WIKIDOT.modules.AccountNotificationsModule = {};

WIKIDOT.modules.AccountNotificationsModule.listeners = {
	loadList: function(e, pageNo){
		var p = null;
		if(pageNo){p = {page: pageNo}};
		OZONE.ajax.requestModule("account/AccountNotificationsListModule", p, WIKIDOT.modules.AccountNotificationsModule.callbacks.loadList);
	}
}

WIKIDOT.modules.AccountNotificationsModule.callbacks = {
	loadList: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		$("notifications-area").innerHTML = r.body;
		OZONE.utils.formatDates($("notifications-area"));
	}
}

WIKIDOT.modules.AccountNotificationsModule.init = function(){
	loadList(1);
}

function loadList(pageNo){
}

WIKIDOT.modules.AccountNotificationsModule.init();
