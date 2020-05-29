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

WIKIDOT.modules.ManageSiteEmailListsModule = {};

WIKIDOT.modules.ManageSiteEmailListsModule.vars = {
	curretStatus: null,
	currentContainer: null,
	currentListId: null
}

WIKIDOT.modules.ManageSiteEmailListsModule.listeners = {
	clickNewList: function(e){
		if(WIKIDOT.modules.ManageSiteEmailListsModule.vars.currentStatus){
			return;
		}
		var form = $('elist-form-template');
		var form2 = form.cloneNode(true);
		var aa = $('elist-action-area');
		// clear aa
		aa.innerHTML = '';
		form2.id="elist-new-list-form";
		aa.appendChild(form2);
		$("elist-add-new-button").style.display="none";
		WIKIDOT.modules.ManageSiteEmailListsModule.vars.currentStatus = 'new';
		
		YAHOO.util.Event.addListener(form2, 'submit', WIKIDOT.modules.ManageSiteEmailListsModule.listeners.saveList);
		
	},
	
	closeEditList: function(e){
		if(WIKIDOT.modules.ManageSiteEmailListsModule.vars.currentStatus == 'edit'){
			var c = WIKIDOT.modules.ManageSiteEmailListsModule.vars.currentContainer;
			c.parentNode.removeChild(c);
			
		}
		if(WIKIDOT.modules.ManageSiteEmailListsModule.vars.currentStatus == 'new'){
			var aa = $('elist-action-area');
			aa.innerHTML = '';
			$("elist-add-new-button").style.display="block";
		}
		WIKIDOT.modules.ManageSiteEmailListsModule.vars = {
			curretStatus: null,
			currentContainer: null,
			currentListId: null
		}
	},
	
	removeList: function(e){
		alert("List removal is not implemented yet.");
	},
	
	editList: function(e, listId){
		if(WIKIDOT.modules.ManageSiteEmailListsModule.vars.currentStatus){
			return;
		}
		var row = $('elist-row-'+listId);
		var isSpecial = YAHOO.util.Dom.hasClass(row, 'elist-special');
		var title = YAHOO.util.Dom.getElementsByClassName('l-title', 'span', row)[0].innerHTML;
		var unixName = YAHOO.util.Dom.getElementsByClassName('l-unixname', 'span', row)[0].innerHTML;
		var whoCanJoin = YAHOO.util.Dom.getElementsByClassName('l-whocanjoin', 'span', row)[0].innerHTML;
		
		// add container
		var tr = document.createElement('tr');
		var td = document.createElement('td');
		tr.appendChild(td);
		td.colSpan = 4;
		
		var form2 = $('elist-form-template').cloneNode(true);
		form2.id="elist-edit";
		var inputs = form2.getElementsByTagName('input');
		inputs[1].value = unixName;
		inputs[0].value = title;
		form2.getElementsByTagName('select')[0].value=whoCanJoin;
		td.appendChild(form2);
		
		if(isSpecial){
			inputs[1].disabled = true;
			form2.getElementsByTagName('select')[0].disabled = true;
		}
		
		OZONE.dom.insertAfter(row.parentNode, tr, row);
		
		WIKIDOT.modules.ManageSiteEmailListsModule.vars.currentStatus = 'edit';
		WIKIDOT.modules.ManageSiteEmailListsModule.vars.currentContainer = tr;
		WIKIDOT.modules.ManageSiteEmailListsModule.vars.currentListId = listId;
		
		YAHOO.util.Event.addListener(form2, 'submit', WIKIDOT.modules.ManageSiteEmailListsModule.listeners.saveList);
		
	},
	
	embedInfo: function(e, listId){
		WIKIDOT.modules.ManageSiteEmailListsModule.listeners.closeEmbedInfo();
		var row = $('elist-row-'+listId);
		// add container
		var tr = document.createElement('tr');
		tr.className = 'elist-embedinfo-row';
		var td = document.createElement('td');
		tr.appendChild(td);
		td.colSpan = 4;
		
		var tt = $('elist-embed-template').cloneNode(true);
		td.appendChild(tt);
		OZONE.dom.insertAfter(row.parentNode, tr, row);
		
		var ll = YAHOO.util.Dom.getElementsByClassName('l-unixname','span', tt)[0];
		var unixName = YAHOO.util.Dom.getElementsByClassName('l-unixname', 'span', row)[0].innerHTML;
		ll.innerHTML=unixName;
	},
	
	closeEmbedInfo: function(e){
		var cs = YAHOO.util.Dom.getElementsByClassName('elist-embedinfo-row');
		for(i = 0; i<cs.length; i++){
			cs[i].parentNode.removeChild(cs[i]);
		}
	},
	
	saveList: function(e){
		var form = this;
		var p = OZONE.utils.formToArray(form);
		if(WIKIDOT.modules.ManageSiteEmailListsModule.vars.currentListId){
			p.listId = WIKIDOT.modules.ManageSiteEmailListsModule.vars.currentListId;
		}
		p.action = 'ManageSiteEmailListsAction';
		p.event  = 'saveList';
		
		OZONE.ajax.requestModule(null, p, WIKIDOT.modules.ManageSiteEmailListsModule.callbacks.saveList);
		
	},
	
	showSubscribers: function(event, listId){
		var p = {};
		p.listId = listId;
		OZONE.ajax.requestModule("managesite/elists/ManageSiteEmailListSubscribersModule", p, WIKIDOT.modules.ManageSiteEmailListsModule.callbacks.showSubscribers);
	},
	
	reloadMain: function(event){
		WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-email-lists');
	},
	
	removeSubscriber: function(e, userId, listId){
		var p = {};
		p.userId = userId;
		p.listId = listId;
		p.action = 'ManageSiteEmailListsAction';
		p.event  = 'unsubscribe';
		OZONE.ajax.requestModule("managesite/elists/ManageSiteEmailListSubscribersModule", p, WIKIDOT.modules.ManageSiteEmailListsModule.callbacks.showSubscribers);
	}
	
}
WIKIDOT.modules.ManageSiteEmailListsModule.callbacks = {
	saveList: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}

		WIKIDOT.modules.ManagerSiteModule.utils.loadModule('sm-email-lists');
	},
	
	showSubscribers: function(r){
		if(!WIKIDOT.utils.handleError(r)) {return;}
		$("sm-action-area").innerHTML = r.body;
	}
}
