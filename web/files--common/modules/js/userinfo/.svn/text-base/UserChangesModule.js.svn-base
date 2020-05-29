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

WIKIDOT.modules.UserChangesModule = {};

WIKIDOT.modules.UserChangesModule.listeners = {
	updateList: function(pageNo){
		var p = new Object();
		if(pageNo != null){
			p.page = pageNo;
		}else{
			p.page = 1;
		}
		p.perpage = $("rev-perpage").value;
		
		p.userId = $("changes-user-id").value;
		
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
		
		//WIKIDOT.modules.PageHistoryModule.vars.params = p; // for pagination
		
		OZONE.ajax.requestModule("userinfo/UserChangesListModule", p, WIKIDOT.modules.UserChangesModule.callbacks.updateList);
	}
}

WIKIDOT.modules.UserChangesModule.callbacks = {
	updateList: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		
		$("site-changes-list").innerHTML = r.body;
		OZONE.utils.formatDates("site-changes-list");
		OZONE.dialog.hovertip.makeTip($("site-changes-list").getElementsByTagName('span'),
				{style: {width: 'auto'}});
	}	
	
}

WIKIDOT.modules.UserChangesModule.init = function(){
	OZONE.dom.onDomReady(function(){
		OZONE.utils.formatDates("site-changes-list");
		OZONE.dialog.hovertip.makeTip($("site-changes-list").getElementsByTagName('span'),
					{style: {width: 'auto'}});
	}, "dummy-ondomready-block");
}

WIKIDOT.modules.UserChangesModule.init();
